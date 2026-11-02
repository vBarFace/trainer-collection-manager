//! Optional manually allocated planning resources, never observed balances or a spending ledger.
use crate::inventory::{Error, InventoryRepository, PokemonId, Result};
use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceValues {
    pub candy: Option<u32>,
    pub candy_xl: Option<u32>,
    pub stardust: Option<u32>,
    pub maximum_level: Option<f64>,
    pub normal_costs_confirmed: bool,
    #[serde(default)]
    pub selected_path_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceBudget {
    pub revision: String,
    #[serde(flatten)]
    pub values: ResourceValues,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            revision: "0".into(),
            values: ResourceValues::default(),
        }
    }
}

impl InventoryRepository {
    pub fn preparation_resources(&self, id: &PokemonId) -> Result<ResourceBudget> {
        self.get(id)?;
        let text: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM application_settings WHERE key=?1",
                [format!("preparation-resources:{}", id.as_str())],
                |row| row.get(0),
            )
            .optional()?;
        let result: ResourceBudget = match text {
            None => ResourceBudget::default(),
            Some(text) if text.len() <= 4096 => serde_json::from_str(&text)
                .map_err(|_| Error::InvalidInput("Invalid saved preparation resources"))?,
            _ => {
                return Err(Error::InvalidInput(
                    "Preparation resources exceed size limit",
                ))
            }
        };
        validate(&result.values)?;
        revision(&result.revision)?;
        Ok(result)
    }

    pub fn save_preparation_resources(
        &self,
        id: &PokemonId,
        expected_revision: i64,
        values: ResourceValues,
    ) -> Result<ResourceBudget> {
        validate(&values)?;
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        let next = self.save_preparation_resources_in_snapshot(id, expected_revision, values)?;
        transaction.commit()?;
        Ok(next)
    }
    pub(crate) fn save_preparation_resources_in_snapshot(
        &self,
        id: &PokemonId,
        expected_revision: i64,
        values: ResourceValues,
    ) -> Result<ResourceBudget> {
        validate(&values)?;
        let current = self.preparation_resources(id)?;
        if revision(&current.revision)? != expected_revision {
            return Err(Error::ObservationConflict);
        }
        if current.values == values {
            return Ok(current);
        }
        let next = ResourceBudget {
            revision: expected_revision
                .checked_add(1)
                .ok_or(Error::InvalidInput("Resource revision overflow"))?
                .to_string(),
            values,
        };
        let text = serde_json::to_string(&next)
            .map_err(|_| Error::InvalidInput("Unable to encode resources"))?;
        self.connection.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=CURRENT_TIMESTAMP",
            params![format!("preparation-resources:{}",id.as_str()), text])?;
        Ok(next)
    }
}

fn revision(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .ok()
        .filter(|n| *n >= 0 && n.to_string() == value)
        .ok_or(Error::InvalidInput("Invalid resource revision"))
}
fn validate(values: &ResourceValues) -> Result<()> {
    if values
        .selected_path_id
        .as_ref()
        .is_some_and(|id| !crate::reference::valid_text(id, 128))
    {
        return Err(Error::InvalidInput("Invalid selected preparation path"));
    }
    if values.maximum_level.is_some_and(|level| {
        !level.is_finite() || !(1.0..=50.0).contains(&level) || (level * 2.0).fract() != 0.0
    }) {
        return Err(Error::InvalidInput(
            "Power-up cap must be an in-game confirmed half level between 1 and 50",
        ));
    }
    Ok(())
}
