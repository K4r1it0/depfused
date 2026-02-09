import React from "react";
import { Button } from "@xq9zk7823/design-system";
import { trackEvent } from "@xq9zk7823/analytics-tracker";
import Link from "next/link";

export default function Sidebar() {
  const navItems = [
    { href: "/", label: "Home" },
    { href: "/about", label: "About" },
    { href: "/dashboard", label: "Dashboard" },
  ];

  return (
    <nav style={{ width: "200px", padding: "1rem", borderRight: "1px solid #ccc" }}>
      <h3>Navigation</h3>
      <ul style={{ listStyle: "none", padding: 0 }}>
        {navItems.map(item => (
          <li key={item.href} style={{ marginBottom: "0.5rem" }}>
            <Link href={item.href}>
              <Button variant="link" onClick={() => trackEvent("nav_click", { to: item.href })}>
                {item.label}
              </Button>
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}
