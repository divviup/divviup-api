import { Outlet } from "react-router";
import Container from "react-bootstrap/Container";
import Header, { HeaderPlaceholder } from "./Header";

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
