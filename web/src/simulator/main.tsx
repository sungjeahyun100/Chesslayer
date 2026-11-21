import React from "react";
import { createRoot } from "react-dom/client";
import { SimulatorApp } from "./SimulatorApp";
import "../style.css";
import "./simulator.css";

createRoot(document.getElementById("root")!).render(<React.StrictMode><SimulatorApp /></React.StrictMode>);
