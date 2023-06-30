import Container from "react-bootstrap/Container";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React from "react";
import { useLoaderData, useAsyncValue, Await } from "react-router-dom";
import { Account } from "./ApiClient";
import ListGroup from "react-bootstrap/ListGroup";
import { LinkContainer } from "react-router-bootstrap";
import { Button } from "react-bootstrap";
import { BuildingAdd } from "react-bootstrap-icons";

function Breadcrumbs() {
  return (
    <Row>
      <Col>
        <Breadcrumb>
          <LinkContainer to="/">
            <Breadcrumb.Item>Home</Breadcrumb.Item>
          </LinkContainer>
          <Breadcrumb.Item active>Accounts</Breadcrumb.Item>
        </Breadcrumb>
      </Col>
    </Row>
  );
}

export default function AccountList() {
  let { accounts } = useLoaderData() as { accounts: Promise<Account[]> };
  return (
    <Container>
      <Breadcrumbs />
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
                <ListGroup.Item>Loading</ListGroup.Item>
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
  let tasks = useAsyncValue() as Account[];
  return (
    <ListGroup>
      {tasks.map((t) => (
        <LinkContainer to={`/accounts/${t.id}`} key={t.id}>
          <ListGroup.Item action>{t.name}</ListGroup.Item>
        </LinkContainer>
      ))}
    </ListGroup>
  );
}
