import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

const handlers = [
  http.get("http://localhost:3000/api_url", () => {
    return HttpResponse.text("http://api.invalid/");
  }),
  http.get("http://api.invalid/api/users/me", () => {
    return HttpResponse.json({
      admin: false,
    });
  }),
  http.get("http://api.invalid/api/accounts", () => {
    return HttpResponse.json([
      {
        name: "Test account",
        id: "00000000-0000-0000-0000-000000000000",
      },
    ]);
  }),
];

const server = setupServer(...handlers);

export const testApiMock = test.extend("server", server);
testApiMock.beforeAll(() => server.listen());
testApiMock.afterEach(() => server.resetHandlers());
testApiMock.afterAll(() => server.close());
