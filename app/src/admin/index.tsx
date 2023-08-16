import { RouteObject } from "react-router-dom";
import ApiClient from "../ApiClient";

export default function admin(
  _apiClient: ApiClient,
  children: RouteObject[],
): RouteObject {
  return {
    path: "admin",
    children,
  };
}
