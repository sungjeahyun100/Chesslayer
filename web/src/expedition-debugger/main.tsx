import React from "react";
import { createRoot } from "react-dom/client";
import { ExpeditionDebugger } from "./ExpeditionDebugger";
import "../style.css";
import "./debugger.css";

createRoot(document.getElementById("root")!).render(<React.StrictMode><ExpeditionDebugger /></React.StrictMode>);
