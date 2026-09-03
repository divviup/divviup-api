import { render, screen } from "@testing-library/react";
import { testApiMock } from "./mocks.js";

testApiMock("smoke test of accounts list via router", async () => {
  // We make our first request for /api_url while importing ApiClientContext, so
  // we need to delay importing this module until after test mocks have been set
  // up.
  const appModule = await import("./App.js");
  const App = appModule.default;
  render(<App />);
  expect(await screen.findByText("Test account")).toBeTruthy();
});
