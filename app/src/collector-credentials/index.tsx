import { RouteObject } from "react-router";
import ApiClient from "../ApiClient";
import CollectorCredentials from "./CollectorCredentialList";
export default function collectorCredentials(
  apiClient: ApiClient,
): RouteObject {
  return {
    path: "collector_credentials",
    children: [
      {
        path: "",
        index: true,
        element: <CollectorCredentials />,
        loader({ params }) {
          return {
            collectorCredentials: apiClient.accountCollectorCredentials(
              params.accountId as string,
            ),
          };
        },

        id: "collectorCredentials",

        shouldRevalidate(_) {
          return true;
        },

        async action({ params, request }) {
          switch (request.method) {
            case "POST":
              return await apiClient.createCollectorCredential(
                params.accountId as string,
                Object.fromEntries(await request.formData()) as {
                  name: string;
                  hpke_config: string;
                },
              );
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },

      {
        path: ":apiTokenId",
        async action({ params, request }) {
          switch (request.method) {
            case "PATCH":
              await apiClient.updateCollectorCredential(
                params.apiTokenId as string,
                Object.fromEntries(await request.formData()) as {
                  name: string;
                },
              );
              return true;
            case "DELETE":
              await apiClient.deleteCollectorCredential(
                params.apiTokenId as string,
              );
              return true;
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },
    ],
  };
}
