import React from "react";
import ReactDOM from "react-dom/client";
//import reportWebVitals from "./reportWebVitals";
import "bootstrap/dist/css/bootstrap.min.css";
import { ApiClientContext } from "./ApiClientContext";
import { ApiClient } from "./ApiClient";
import Router from "./router";
import { PrimeReactProvider } from "primereact/api";
import "primereact/resources/themes/lara-light-indigo/theme.css";
import "primereact/resources/primereact.min.css";

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement,
);

function App() {
  const apiClient = React.useMemo(() => new ApiClient(), []);
  return (
    <ApiClientContext.Provider value={apiClient}>
      <PrimeReactProvider>
        <Router />
      </PrimeReactProvider>
    </ApiClientContext.Provider>
  );
}

root.render(<App />);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
//reportWebVitals(console.log);
