import { useState, useEffect } from "react";
import { MENU_ITEMS, MenuItem } from "../../constants";
import { invoke } from "@tauri-apps/api/core";

import "../../App.css";
import "./Menu.css";

export default function Menu() {
  const [expandedItems, setExpandedItems] = useState<string[]>(["workspace"]);
  const [branches, setBranches] = useState<MenuItem[]>([]);

  useEffect(() => {
    invoke<{ name: string }[]>("branches")
      .then((branchList: any) => {
        const branchMenuItems: MenuItem[] = branchList.map(
          (branch: { is_current: boolean; name: string }) => ({
            id: `branch-${branch.name}`,
            label: branch.is_current ? `* ${branch.name}` : branch.name,
          })
        );
        setBranches(branchMenuItems);
      })
      .catch(console.error);
  }, []);

  const toggleItem = (id: string) => {
    setExpandedItems((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  };

  const renderMenuItem = (item: MenuItem, isChild = false) => (
    <li key={item.id} className={isChild ? "menu-child-item" : "menu-item"}>
      {item.children !== undefined ? (
        <>
          <button
            onClick={() => toggleItem(item.id)}
            className={`menu-button ${isChild ? "child" : "parent"}`}
          >
            {!isChild && item.children.length > 0 && (
              <span className="toggle-icon">
                {expandedItems.includes(item.id) ? "⌄" : "›"}
              </span>
            )}
            {item.icon && <img className="icon" src={item.icon} alt="" />}
            <span className="label">{item.label}</span>
          </button>

          {expandedItems.includes(item.id) && item.children.length > 0 && (
            <ul className="submenu">
              {item.children.map((child) => renderMenuItem(child, true))}
            </ul>
          )}
        </>
      ) : (
        <div className="menu-button child">
          {item.icon && <img className="icon" src={item.icon} alt="" />}
          <span className="label">{item.label}</span>
        </div>
      )}
    </li>
  );

  const finalMenu = MENU_ITEMS.map((item) => {
    if (item.id === "branches") {
      return { ...item, children: branches };
    }
    return item;
  });

  return (
    <nav className="sidebar-menu">
      <div className="sidebar-header">
        <div className="app-title">flux</div>
        <div className="app-subtitle">Version Control System</div>
      </div>

      <ul className="menu-list">
        {finalMenu.map((item) => renderMenuItem(item))}
      </ul>
    </nav>
  );
}
