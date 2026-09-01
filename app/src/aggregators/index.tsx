import Aggregators from "./AggregatorList.js";
import AggregatorFormPage from "./AggregatorForm.js";
import AggregatorDetail from "./AggregatorDetail.js";
import ApiClient from "../ApiClient.js";
import { RouteObject, redirect } from "react-router-dom";

export default function aggregators(apiClient: ApiClient): RouteObject {
  return {
    path: "aggregators",
    children: [
      {
        path: "",
        index: true,
        element: <Aggregators />,
        loader({ params }) {
          return {
            aggregators: apiClient.accountAggregators(
              params.accountId as string,
            ),
          };
        },
      },
      {
        path: ":aggregatorId",
        element: <AggregatorDetail />,
        loader({ params }) {
          return {
            aggregator: apiClient.aggregator(params.aggregatorId as string),
          };
        },

        async action({ params, request }) {
          const data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "PATCH":
              return await apiClient.updateAggregator(
                params.aggregatorId as string,
                data as { name: string } | { bearer_token: string },
              );
            case "DELETE":
              await apiClient.deleteAggregator(params.aggregatorId as string);
              return redirect("..");
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },

      {
        path: "new",
        element: <AggregatorFormPage />,
      },
    ],
  };
}
