import { RouteObject, defer } from "react-router-dom";
import ApiClient from "../ApiClient";
import ApiTokenList from "./ApiTokenList";
export default function apiTokens(apiClient: ApiClient): RouteObject {
  return {
    path: "api_tokens",
    children: [
      {
        path: "",
        index: true,
        element: <ApiTokenList />,
        loader({ params }) {
          return defer({
            apiTokens: apiClient
              .accountApiTokens(params.account_id as string)
              .then((tokens) => tokens.reverse()),
          });
        },

        shouldRevalidate(_) {
          return true;
        },

        async action({ params, request }) {
          switch (request.method) {
            case "POST":
              return await apiClient.createApiToken(
                params.account_id as string
              );
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },

      {
        path: ":api_token_id",
        async action({ params, request }) {
          switch (request.method) {
            case "PATCH":
              await apiClient.updateApiToken(
                params.api_token_id as string,
                Object.fromEntries(await request.formData()) as {
                  name: string;
                }
              );
              return true;
            case "DELETE":
              await apiClient.deleteApiToken(params.api_token_id as string);
              return true;
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },
    ],
  };
}
