import { render, screen } from "@testing-library/react";
import Router from "./router.js";
test("tests have not been written yet but we need at least one test", () => {
  render(<Router />);
  expect(screen.queryByText("test")).toBeNull();
});
