import { useState } from "react";
import { Inventory } from "./Inventory";
import { ReferenceSettings } from "./ReferenceSettings";
import { Recommendations } from "./Recommendations";

const navigationItems = ["Inventory", "Recommendations", "Settings"] as const;
type NavigationItem = (typeof navigationItems)[number];

export function App() {
  const [activeItem, setActiveItem] = useState<NavigationItem>("Inventory");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main-content">
        Skip to content
      </a>
      <aside className="sidebar">
        <div className="brand">
          <span aria-hidden="true">◈</span> Trainer Collection Manager
        </div>
        <nav aria-label="Primary navigation">
          <ul>
            {navigationItems.map((item) => (
              <li key={item}>
                <button
                  type="button"
                  className={
                    item === activeItem ? "nav-item active" : "nav-item"
                  }
                  aria-current={item === activeItem ? "page" : undefined}
                  onClick={() => setActiveItem(item)}
                >
                  {item}
                </button>
              </li>
            ))}
          </ul>
        </nav>
        <p className="privacy-note">
          Local-first. No account, cloud sync, telemetry, or game automation.
        </p>
      </aside>
      <main id="main-content">
        <p className="eyebrow">Offline collection workspace · MVP preview</p>
        <h1>{activeItem}</h1>
        {activeItem === "Inventory" ? (
          <Inventory
            initialSelectedId={selectedId}
            onViewAssessment={(id) => {
              setSelectedId(id);
              setActiveItem("Recommendations");
            }}
          />
        ) : activeItem === "Settings" ? (
          <ReferenceSettings />
        ) : (
          <Recommendations
            initialSelectedId={selectedId}
            onOpenIndividual={(id) => {
              setSelectedId(id);
              setActiveItem("Inventory");
            }}
          />
        )}
      </main>
    </div>
  );
}
