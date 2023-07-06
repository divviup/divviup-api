import { RouteObject, redirect } from "react-router-dom";
import ApiClient from "../ApiClient";
import ErrorPage from "./ErrorPage";
import { AxiosError } from "axios";
import Layout from "./Layout";

export default function layout(
  apiClient: ApiClient,
  children: RouteObject[]
): RouteObject {
  return {
    path: "/",
    element: <Layout />,
    id: "currentUser",
    async loader() {
      try {
        const currentUser = await apiClient.getCurrentUser();
        return { currentUser };
      } catch (e) {
        if (e instanceof AxiosError && e.response?.status === 403) {
          return await apiClient.redirectToLogin();
        } else throw e;
      }
    },
    shouldRevalidate(_) {
      return false;
    },
    errorElement: <ErrorPage apiClient={apiClient} />,
    children,
  };
}
