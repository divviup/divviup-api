import React from "react";
import { ApiClient } from "./ApiClient.js";
export const ApiClientContext = React.createContext<ApiClient>(new ApiClient());
