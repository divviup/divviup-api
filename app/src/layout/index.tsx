import { RouteObject } from "react-router-dom";
import ApiClient from "../ApiClient.js";
import ErrorPage from "./ErrorPage.js";
import Layout from "./Layout.js";
export default function layout(
  apiClient: ApiClient,
  children: RouteObject[],
): RouteObject {
  return {
    path: "/",
    element: <Layout />,
    id: "currentUser",
    async loader() {
      const currentUser = await apiClient.currentUser();
      return { currentUser };
    },
    shouldRevalidate(_) {
      return false;
    },
    errorElement: <ErrorPage apiClient={apiClient} />,
    children,
  };
}
