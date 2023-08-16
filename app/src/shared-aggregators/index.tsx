import { RouteObject, defer } from "react-router-dom";
import ApiClient, { UpdateAggregator } from "../ApiClient";

export default function sharedAggregators(apiClient: ApiClient): RouteObject {
  return {
    path: "aggregators",
    children: [
      {
        path: "",
        index: true,
        async lazy() {
          return import("./SharedAggregatorList");
        },
        async loader() {
          return defer({ aggregators: apiClient.sharedAggregators() });
        },
      },
      {
        path: ":aggregator_id",
        async action({ request, params }) {
          switch (request.method) {
            case "DELETE":
              await apiClient.deleteAggregator(params.aggregator_id as string);
              return null;
            case "PATCH":
              return apiClient.updateAggregator(
                params.aggregator_id as string,
                Object.fromEntries(
                  await request.formData(),
                ) as UpdateAggregator,
              );
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },
    ],
  };
}
