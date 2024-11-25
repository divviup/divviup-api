import { RouteObject, redirect } from "react-router";
import ApiClient, { NewAccount, UpdateAccount } from "../ApiClient";
import AccountSummary from "./AccountSummary";
import AccountForm from "./AccountForm";
import AccountList from "./AccountList";

export default function accounts(
  apiClient: ApiClient,
  children: RouteObject[],
): RouteObject {
  return {
    path: "accounts",

    children: [
      {
        path: "",
        element: <AccountList />,
        loader() {
          return { accounts: apiClient.accounts() };
        },
        index: true,
      },
      {
        path: "new",
        element: <AccountForm />,
        async action({ request }) {
          const data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "POST": {
              const account = await apiClient.createAccount(
                data as unknown as NewAccount,
              );
              return redirect(`/accounts/${account.id}`);
            }
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },

      {
        path: ":accountId",
        id: "account",
        loader({ params }) {
          const { accountId } = params as { accountId: string };
          return {
            account: apiClient.account(accountId),
          };
        },
        shouldRevalidate(args) {
          return (
            typeof args.actionResult === "object" &&
            args.actionResult !== null &&
            "account" in args.actionResult
          );
        },

        async action({ params, request }) {
          switch (request.method) {
            case "PATCH": {
              const data = await request.json();
              return {
                account: await apiClient.updateAccount(
                  params.accountId as string,
                  data as UpdateAccount,
                ),
              };
            }
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },

        children: [
          {
            element: <AccountSummary />,
            path: "",
            index: true,
            loader({ params }) {
              const { accountId } = params as { accountId: string };
              return {
                apiTokens: apiClient.accountApiTokens(accountId),
                tasks: apiClient.accountTasks(accountId),
                collectorCredentials:
                  apiClient.accountCollectorCredentials(accountId),
                aggregators: apiClient.accountAggregators(accountId),
                account: apiClient.account(accountId),
              };
            },
          },
          ...children,
        ],
      },
    ],
  };
}
