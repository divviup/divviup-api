import { RouteObject, defer } from "react-router-dom";
import ApiClient from "../ApiClient";
import HpkeConfigList from "./HpkeConfigList";
export default function apiTokens(apiClient: ApiClient): RouteObject {
  return {
    path: "hpke_configs",
    children: [
      {
        path: "",
        index: true,
        element: <HpkeConfigList />,
        loader({ params }) {
          return defer({
            hpkeConfigs: apiClient.accountHpkeConfigs(
              params.accountId as string,
            ),
          });
        },

        id: "hpkeConfigs",

        shouldRevalidate(_) {
          return true;
        },

        async action({ params, request }) {
          switch (request.method) {
            case "POST":
              return await apiClient.createHpkeConfig(
                params.accountId as string,
                Object.fromEntries(await request.formData()) as {
                  name: string;
                  contents: string;
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
              await apiClient.updateHpkeConfig(
                params.apiTokenId as string,
                Object.fromEntries(await request.formData()) as {
                  name: string;
                },
              );
              return true;
            case "DELETE":
              await apiClient.deleteHpkeConfig(params.apiTokenId as string);
              return true;
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },
    ],
  };
}
