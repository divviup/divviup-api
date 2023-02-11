import React from "react";
import { ApiClient } from "./ApiClient";
export const ApiClientContext = React.createContext<ApiClient>(new ApiClient());
