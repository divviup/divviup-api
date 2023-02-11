import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import {
  Await,
  Form,
  useActionData,
  useAsyncValue,
  useRouteLoaderData,
} from "react-router-dom";
import { Suspense, useCallback, useEffect, useState } from "react";
import { Account } from "./ApiClient";

import Spinner from "react-bootstrap/Spinner";
import { LinkContainer } from "react-router-bootstrap";
import {
  Building,
  FileEarmarkCode,
  PencilFill,
  People,
} from "react-bootstrap-icons";
import { Button, FormControl, InputGroup } from "react-bootstrap";
export default function AccountSummary() {
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  return (
    <Suspense fallback={<Spinner />}>
      <Await resolve={account}>
        <AccountSummaryLoaded />
      </Await>
    </Suspense>
  );
}

function AccountName({ account }: { account: Account }) {
  let [isEditingName, setIsEditingName] = useState(false);
  let edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  let actionData = useActionData();
  useEffect(() => {
    if (actionData) setIsEditingName(false);
  }, [actionData]);

  if (isEditingName) {
    return (
      <>
        <Row>
          <Col xs="11">
            <Form method="patch">
              <InputGroup hasValidation>
                <InputGroup.Text id="inputGroupPrepend">
                  <Building />
                </InputGroup.Text>
                <FormControl
                  type="text"
                  name="name"
                  defaultValue={account.name}
                  required
                />
              </InputGroup>
            </Form>
          </Col>
          <Col>
            <Button variant="primary" type="submit">
              <PencilFill />
            </Button>
          </Col>
        </Row>
      </>
    );
  } else {
    return (
      <Row>
        <Col xs="11">
          <h1>
            <Building />
            {account.name}
          </h1>
        </Col>
        <Col>
          <Button variant="secondary" onClick={edit}>
            <PencilFill />
          </Button>
        </Col>
      </Row>
    );
  }
}

export function AccountSummaryLoaded() {
  let account = useAsyncValue() as Account;
  return (
    <>
      <Row>
        <Col>
          <Breadcrumb>
            <LinkContainer to="/">
              <Breadcrumb.Item>Home</Breadcrumb.Item>
            </LinkContainer>
            <LinkContainer to="/accounts">
              <Breadcrumb.Item>Accounts</Breadcrumb.Item>
            </LinkContainer>
            <Breadcrumb.Item active>{account.name}</Breadcrumb.Item>
          </Breadcrumb>
        </Col>
      </Row>
      <Row>
        <Col>
          <AccountName account={account} />
          <ListGroup>
            <LinkContainer to="memberships">
              <ListGroup.Item action>
                <People /> Members
              </ListGroup.Item>
            </LinkContainer>

            <LinkContainer to="tasks">
              <ListGroup.Item action>
                <FileEarmarkCode /> Tasks
              </ListGroup.Item>
            </LinkContainer>
          </ListGroup>
        </Col>
      </Row>
    </>
  );
}
