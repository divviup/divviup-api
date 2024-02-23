import { RouteObject } from "react-router-dom";

export default function swaggerUi(): RouteObject {
  return {
    path: "swagger-ui",
    async lazy() {
      const { default: SwaggerUI } = await import("swagger-ui-react");
      await import("swagger-ui-react/swagger-ui.css");
      return {
        Component() {
          return (
            <SwaggerUI
              url="/swagger.yml"
              supportedSubmitMethods={[]}
              deepLinking={true}
            />
          );
        },
      };
    },
  };
}
