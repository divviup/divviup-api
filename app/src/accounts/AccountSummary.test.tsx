import { render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import AccountSummary from "./AccountSummary.js";
import {
  Account,
  Aggregator,
  ApiToken,
  CollectorCredential,
  Task,
} from "../ApiClient.js";

test("AccountSummary renders", async () => {
  const router = createMemoryRouter(
    [
      {
        path: "/accounts/:account_id",
        id: "account",
        element: <AccountSummary />,
        async loader({ params }) {
          const { accountId } = params as { accountId: string };
          return {
            apiTokens: (async (): Promise<ApiToken[]> => {
              return [
                {
                  id: "00000000-0000-0000-0000-000000000001",
                  account_id: accountId,
                  token_hash: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                  created_at: "1985-04-12T23:20:50.52Z",
                  name: "Test API token",
                },
              ];
            })(),
            tasks: (async (): Promise<Task[]> => {
              return [];
            })(),
            collectorCredentials: (async (): Promise<CollectorCredential[]> => {
              return [];
            })(),
            aggregators: (async (): Promise<Aggregator[]> => {
              return [];
            })(),
            account: (async (): Promise<Account> => {
              return {
                name: "Test account",
                id: accountId,
                created_at: "1985-04-12T23:20:50.52Z",
                updated_at: "1985-04-12T23:20:50.52Z",
                intends_to_use_shared_aggregators: false,
                admin: false,
              };
            })(),
          };
        },
      },
    ],
    {
      initialEntries: ["/accounts/00000000-0000-0000-0000-000000000000"],
    },
  );

  render(<RouterProvider router={router} />);

  await screen.findByText("Test account");
});
