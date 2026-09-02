// jest-dom adds custom jest matchers for asserting on DOM nodes.
// allows you to do things like:
// expect(element).toHaveTextContent(/react/i)
// learn more: https://github.com/testing-library/jest-dom
import "@testing-library/jest-dom/vitest";

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

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
