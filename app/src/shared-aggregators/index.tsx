import { RouteObject } from "react-router-dom";
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
          return { aggregators: apiClient.sharedAggregators() };
        },
      },
      {
        path: ":aggregatorId",
        async action({ request, params }) {
          switch (request.method) {
            case "DELETE":
              await apiClient.deleteAggregator(params.aggregatorId as string);
              return null;
            case "PATCH":
              return apiClient.updateAggregator(
                params.aggregatorId as string,
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
