import {
  Await,
  Link,
  useAsyncValue,
  useRouteLoaderData,
} from "react-router-dom";
import Container from "react-bootstrap/Container";
import Navbar from "react-bootstrap/Navbar";
import { User } from "../ApiClient";
import { Suspense } from "react";
import NavDropdown from "react-bootstrap/NavDropdown";
import logo from "../logo/color/svg/cropped.svg";
import { LinkContainer } from "react-router-bootstrap";
import { Nav } from "react-bootstrap";

function HeaderPlaceholder() {
  return (
    <Navbar bg="light">
      <Container>
        <Navbar.Brand as={Link} to="/">
          <img src={logo} alt="DivviUp" width="100" />
        </Navbar.Brand>
        <LinkContainer to="/login">
          <NavDropdown.Item>Log In</NavDropdown.Item>
        </LinkContainer>
      </Container>
    </Navbar>
  );
}

function LoggedInHeader() {
  let user = useAsyncValue() as User;

  return (
    <Navbar bg="light" expand="lg">
      <Container>
        <Navbar.Brand as={Link} to="/accounts">
          <img src={logo} alt="DivviUp" width="100" />
        </Navbar.Brand>

        {user.admin ? (
          <>
            <Nav className="">
              <LinkContainer to="/admin/queue">
                <Nav.Link>Queue</Nav.Link>
              </LinkContainer>
            </Nav>{" "}
            <Nav className="me-auto">
              <LinkContainer to="/admin/aggregators">
                <Nav.Link>Aggregators</Nav.Link>
              </LinkContainer>
            </Nav>
          </>
        ) : null}

        <NavDropdown title={user.name}>
          <LinkContainer to="/logout">
            <NavDropdown.Item>Log Out</NavDropdown.Item>
          </LinkContainer>
        </NavDropdown>
      </Container>
    </Navbar>
  );
}

export default function Header() {
  let { currentUser } = useRouteLoaderData("currentUser") as {
    currentUser: Promise<User>;
  };
  return (
    <Suspense fallback={<HeaderPlaceholder />}>
      <Await resolve={currentUser}>
        {(user) => (user ? <LoggedInHeader /> : <HeaderPlaceholder />)}
      </Await>
    </Suspense>
  );
}
