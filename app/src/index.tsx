import React, { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "bootstrap/dist/css/bootstrap.min.css";
import "primereact/resources/themes/lara-light-indigo/theme.css";
import "primereact/resources/primereact.min.css";
import { RelativeTimeElement } from "@github/relative-time-element";
import App from "./App.js";

declare module "react" {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    interface IntrinsicElements {
      "relative-time": React.DetailedHTMLProps<
        React.HTMLAttributes<RelativeTimeElement>,
        RelativeTimeElement
      > &
        Partial<Omit<RelativeTimeElement, keyof HTMLElement>>;
    }
  }
}

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement,
);

root.render(
  <StrictMode>
    <App />
  </StrictMode>,
);
