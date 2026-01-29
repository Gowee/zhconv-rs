import { render } from "@testing-library/react";
import { expect, test, describe } from "vitest";

import App from "./App";

describe("App", () => {
  test("renders correctly", () => {
    document.title = "zhs: Online Chinese Converter";
    render(<App />);
    expect(document.title).toBe("zhs: Online Chinese Converter");
  });
});
