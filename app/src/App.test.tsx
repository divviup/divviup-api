import "@testing-library/jest-dom";
import React from "react";
import { render, screen } from "@testing-library/react";
import Router from "./router";
test("tests have not been written yet but we need at least one test", () => {
  render(<Router />);
  expect(screen.queryByText("test")).toBeNull();
});
