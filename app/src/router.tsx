import React from "react";
import {
  createBrowserRouter,
  RouterProvider,
  RouteObject,
  redirect,
} from "react-router-dom";
import { ApiClientContext } from "./ApiClientContext.js";
import { ApiClient } from "./ApiClient.js";
import layout from "./layout/index.js";
import admin from "./admin/index.js";
import memberships from "./memberships/index.js";
import tasks from "./tasks/index.js";
import accounts from "./accounts/index.js";
import apiTokens from "./api-tokens/index.js";
import aggregators from "./aggregators/index.js";
import { Spinner } from "react-bootstrap";
import queue from "./queue/index.js";
import sharedAggregators from "./shared-aggregators/index.js";
import collectorCredentials from "./collector-credentials/index.js";
import swaggerUi from "./swagger-ui.js";

function buildRouter(apiClient: ApiClient) {
  return createBrowserRouter([
    swaggerUi(),
    layout(apiClient, [
      logout(apiClient),
      root(apiClient),
      admin(apiClient, [queue(apiClient), sharedAggregators(apiClient)]),
      accounts(apiClient, [
        aggregators(apiClient),
        apiTokens(apiClient),
        memberships(apiClient),
        tasks(apiClient),
        collectorCredentials(apiClient),
      ]),
    ]),
  ]);
}

export default function Router() {
  const apiClient = React.useContext(ApiClientContext);
  const router = React.useMemo(() => buildRouter(apiClient), [apiClient]);
  return <RouterProvider router={router} />;
}

function root(_apiClient: ApiClient): RouteObject {
  return {
    path: "",
    async loader() {
      return redirect("/accounts");
    },
    index: true,
  };
}

function logout(apiClient: ApiClient): RouteObject {
  return {
    path: "logout",
    element: <Spinner />,
    async loader() {
      window.location.href = await apiClient.logoutUrl();
      return null;
    },
  };
}
