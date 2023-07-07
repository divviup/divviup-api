import { RouteObject, defer } from "react-router-dom";
import ApiClient from "../ApiClient";

export default function sharedAggregators(apiClient: ApiClient): RouteObject {
  return {
    path: "aggregators",
    async lazy() {
      return import("./SharedAggregatorList");
    },
    async loader() {
      return defer({ aggregators: apiClient.sharedAggregators() });
    },
  };
}
