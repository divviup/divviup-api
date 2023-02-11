import { Outlet } from "react-router-dom";
import Container from "react-bootstrap/Container";
import Header from "./Header";

export default function Layout({ children }: React.PropsWithChildren) {
  return (
    <main>
      <Header />
      <Container>
        {children}
        <Outlet />
      </Container>
    </main>
  );
}
