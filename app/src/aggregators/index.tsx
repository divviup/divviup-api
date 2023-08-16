import Aggregators from "./AggregatorList";
import AggregatorForm from "./AggregatorForm";
import AggregatorDetail from "./AggregatorDetail";
import ApiClient from "../ApiClient";
import { RouteObject, defer, redirect } from "react-router-dom";

export default function aggregators(apiClient: ApiClient): RouteObject {
  return {
    path: "aggregators",
    children: [
      {
        path: "",
        index: true,
        element: <Aggregators />,
        loader({ params }) {
          return defer({
            aggregators: apiClient.accountAggregators(
              params.account_id as string,
            ),
          });
        },
      },
      {
        path: ":aggregator_id",
        element: <AggregatorDetail />,
        loader({ params }) {
          return defer({
            aggregator: apiClient.aggregator(params.aggregator_id as string),
          });
        },

        async action({ params, request }) {
          const data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "PATCH":
              return await apiClient.updateAggregator(
                params.aggregator_id as string,
                data as { name: string } | { bearer_token: string },
              );
            case "DELETE":
              await apiClient.deleteAggregator(params.aggregator_id as string);
              return redirect("..");
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },

      {
        path: "new",
        element: <AggregatorForm />,
      },
    ],
  };
}
