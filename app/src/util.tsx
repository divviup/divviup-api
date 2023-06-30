import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Suspense } from "react";
import { Await, useRouteLoaderData } from "react-router-dom";
import { Account } from "./ApiClient";

export function WithAccount({
  children,
}: {
  children: (data: Awaited<Account>) => React.ReactNode;
}) {
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };

  return <Await resolve={account} children={children} />;
}

export function AccountBreadcrumbs({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Row>
      <Col>
        <Breadcrumb>
          <LinkContainer to="/">
            <Breadcrumb.Item>Home</Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to="/accounts">
            <Breadcrumb.Item>Accounts</Breadcrumb.Item>
          </LinkContainer>
          <Suspense fallback="...">
            <WithAccount>
              {(account) => (
                <LinkContainer to={`/accounts/${account.id}`}>
                  <Breadcrumb.Item>{account.name}</Breadcrumb.Item>
                </LinkContainer>
              )}
            </WithAccount>
          </Suspense>
          {children}
        </Breadcrumb>
      </Col>
    </Row>
  );
}
