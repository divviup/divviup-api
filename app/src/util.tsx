import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Suspense } from "react";
import { Await, useRouteLoaderData } from "react-router-dom";
import { Account } from "./ApiClient";
import Placeholder from "react-bootstrap/Placeholder";

export function WithAccount({
  children,
}: {
  children: (data: Awaited<Account>) => React.ReactNode;
}) {
  const { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };

  return <Await resolve={account}>{children}</Await>;
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
          <LinkContainer to="/accounts">
            <Breadcrumb.Item>Accounts</Breadcrumb.Item>
          </LinkContainer>
          <Suspense
            fallback={
              <Breadcrumb.Item>
                <Placeholder size="sm" xs={2} />
              </Breadcrumb.Item>
            }
          >
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
