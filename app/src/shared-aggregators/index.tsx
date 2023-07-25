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
    children: [
      {
        path: ":aggregator_id",
        async action({ request, params }) {
          switch (request.method) {
            case "DELETE":
              await apiClient.deleteAggregator(params.aggregator_id as string);
              return null;
            default:
              throw new Error(`unexpected method {request.method}`);
          }
        },
      },
    ],
  };
}
