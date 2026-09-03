import React from "react";
import { ApiClientContext } from "./ApiClientContext.js";
import { ApiClient } from "./ApiClient.js";
import Router from "./router.js";
import { PrimeReactProvider } from "primereact/api";

export default function App() {
  const apiClient = React.useMemo(() => new ApiClient(), []);
  return (
    <ApiClientContext.Provider value={apiClient}>
      <PrimeReactProvider>
        <Router />
      </PrimeReactProvider>
    </ApiClientContext.Provider>
  );
}
