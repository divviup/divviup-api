import React from "react";
import {
  createBrowserRouter,
  RouterProvider,
  RouteObject,
  redirect,
} from "react-router-dom";
import { ApiClientContext } from "./ApiClientContext";
import { ApiClient } from "./ApiClient";
import layout from "./layout";
import admin from "./admin";
import memberships from "./memberships";
import tasks from "./tasks";
import accounts from "./accounts";
import apiTokens from "./api-tokens";
import aggregators from "./aggregators";
import { Spinner } from "react-bootstrap";

function buildRouter(apiClient: ApiClient) {
  return createBrowserRouter([
    layout(apiClient, [
      logout(apiClient),
      root(apiClient),
      admin(apiClient),
      accounts(apiClient, [
        aggregators(apiClient),
        apiTokens(apiClient),
        memberships(apiClient),
        tasks(apiClient),
      ]),
    ]),
  ]);
}

export default function Router() {
  let apiClient = React.useContext(ApiClientContext);
  let router = React.useMemo(() => buildRouter(apiClient), [apiClient]);
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
