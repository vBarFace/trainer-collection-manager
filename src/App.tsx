import { useState } from "react";
import { Inventory } from "./Inventory";

const navigationItems = [
  "Inventory",
  "Recommendations",
  "Teams",
  "Bag",
  "Settings",
] as const;
type NavigationItem = (typeof navigationItems)[number];

const descriptions: Record<NavigationItem, string> = {
  Inventory: "Review screenshots manually and save your local collection.",
  Recommendations:
    "Explainable, deterministic recommendations are planned for a later milestone.",
  Teams:
    "Team building and battle planning are not implemented in this foundation release.",
  Bag: "Item tracking is planned; no game data is collected or inferred yet.",
  Settings:
    "Application preferences will remain local to this Windows user profile.",
};

export function App() {
  const [activeItem, setActiveItem] = useState<NavigationItem>("Inventory");

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
        <p className="eyebrow">Foundation release</p>
        <h1>{activeItem}</h1>
        {activeItem === "Inventory" ? (
          <Inventory />
        ) : (
          <section className="placeholder" aria-labelledby="placeholder-title">
            <p className="status">Planned</p>
            <h2 id="placeholder-title">This area is a placeholder</h2>
            <p>{descriptions[activeItem]}</p>
            <p>
              The app database initializes locally at startup with versioned
              migrations only.
            </p>
          </section>
        )}
      </main>
    </div>
  );
}
