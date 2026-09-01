import { Outlet } from "react-router-dom";
import { Container } from "react-bootstrap";
import Header, { HeaderPlaceholder } from "./Header.js";

export default function Layout({
  error = false,
  children,
}: React.PropsWithChildren & { error?: boolean }) {
  return (
    <main>
      {error ? <HeaderPlaceholder /> : <Header />}
      <Container>
        {children}
        <Outlet />
      </Container>
    </main>
  );
}
