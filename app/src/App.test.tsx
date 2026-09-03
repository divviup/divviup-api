import { render, screen } from "@testing-library/react";

test("smoke test of accounts list via router", async () => {
  // We make our first request for /api_url while importing ApiClientContext, so
  // we need to delay importing this module until after test mocks have been set
  // up.
  const routerModule = await import("./router.js");
  const Router = routerModule.default;
  render(<Router />);
  expect(await screen.findByText("Test account")).toBeTruthy();
});
