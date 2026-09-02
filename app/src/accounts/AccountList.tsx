import React from "react";
import { useLoaderData, useAsyncValue, Await } from "react-router-dom";
import { Account } from "../ApiClient.js";
import { LinkContainer } from "../LinkContainer.js";
import {
  Button,
  Col,
  Container,
  ListGroup,
  Placeholder,
  Row,
} from "react-bootstrap";
import { BuildingAdd } from "react-bootstrap-icons";

export default function AccountList() {
  const { accounts } = useLoaderData() as { accounts: Promise<Account[]> };
  return (
    <Container>
      <Row>
        <Col>
          <h1>Accounts</h1>
        </Col>
      </Row>
      <Row>
        <Col>
          <LinkContainer to="/accounts/new">
            <Button>
              <BuildingAdd /> New
            </Button>
          </LinkContainer>
        </Col>
      </Row>
      <Row>
        <Col>
          <React.Suspense
            fallback={
              <ListGroup>
                <ListGroup.Item>
                  <Placeholder animation="glow" xs={12} />
                </ListGroup.Item>
              </ListGroup>
            }
          >
            <Await resolve={accounts}>
              <LoadedAccounts />
            </Await>
          </React.Suspense>
        </Col>
      </Row>
    </Container>
  );
}

function LoadedAccounts() {
  const accounts = useAsyncValue() as Account[];
  return (
    <ListGroup>
      {accounts.map((t) => (
        <LinkContainer to={`/accounts/${t.id}`} key={t.id}>
          <ListGroup.Item action>{t.name}</ListGroup.Item>
        </LinkContainer>
      ))}
    </ListGroup>
  );
}
