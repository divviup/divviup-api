import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import {
  Await,
  useLoaderData,
  useAsyncValue,
  useRouteLoaderData,
  Form,
  useSubmit,
} from "react-router-dom";
import React, { Suspense, useState } from "react";
import { Account, Membership, User } from "./ApiClient";
import { LinkContainer } from "react-router-bootstrap";
import { Button, FormControl, Spinner } from "react-bootstrap";
import { PersonSlash, PersonAdd, People } from "react-bootstrap-icons";
import Modal from "react-bootstrap/Modal";

export default function Memberships() {
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  return (
    <Suspense fallback={<Spinner />}>
      <Await resolve={account}>
        <MembershipsFull />
      </Await>
    </Suspense>
  );
}

function Breadcrumbs({ account }: { account: Account }) {
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
          <LinkContainer to={`/accounts/${account.id}`}>
            <Breadcrumb.Item>{account.name}</Breadcrumb.Item>
          </LinkContainer>
          <Breadcrumb.Item active>Members</Breadcrumb.Item>
        </Breadcrumb>
      </Col>
    </Row>
  );
}

function MembershipsFull() {
  let account = useAsyncValue() as Account;
  let { memberships } = useLoaderData() as {
    memberships: Promise<Membership[]>;
  };
  return (
    <>
      <Breadcrumbs account={account} />
      <Row>
        <Col>
          <Suspense fallback={<Spinner />}>
            <Await resolve={memberships}>
              <MembershipList account={account} />
            </Await>
          </Suspense>
        </Col>
      </Row>
    </>
  );
}

function DeleteMembershipButton({ membership }: { membership: Membership }) {
  let submit = useSubmit();
  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);

  let deleteMembership = React.useCallback(() => {
    submit({ membershipId: membership.id }, { method: "delete" });
  }, [membership, submit]);

  return (
    <>
      <Button variant="outline-danger" className="ml-auto" onClick={open}>
        <PersonSlash />
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>Confirm Membership Removal</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This user will no longer be able to view or create tasks on this
          account
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <Button variant="primary" onClick={deleteMembership}>
            Remove {membership.user_email}
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
}

function AddMembershipForm() {
  let [email, setEmail] = React.useState("");

  return (
    <Form
      action="."
      method="post"
      onSubmit={React.useCallback(() => {
        setEmail("");
      }, [setEmail])}
    >
      <Row className="my-3">
        <Col xs="11">
          <FormControl
            type="email"
            name="user_email"
            id="user_email"
            value={email}
            autoComplete="off"
            onChange={React.useCallback(
              (event: React.ChangeEvent<HTMLInputElement>) =>
                setEmail(event.target.value),
              [setEmail]
            )}
          />
        </Col>
        <Col>
          <Button variant="primary" type="submit">
            <PersonAdd /> Add
          </Button>
        </Col>
      </Row>
    </Form>
  );
}

function MembershipList({ account }: { account: Account }) {
  let memberships = useAsyncValue() as Membership[];
  let { currentUser } = useRouteLoaderData("currentUser") as {
    currentUser: Promise<User>;
  };

  return (
    <React.Suspense>
      <Await resolve={currentUser}>
        {(currentUser) => (
          <>
            <h3>
              <People /> {account.name} Members
            </h3>
            <AddMembershipForm />
            <ListGroup>
              {memberships.map((membership) => {
                const isCurrent = membership.user_email === currentUser.email;
                return (
                  <ListGroup.Item
                    className="d-flex justify-content-between align-items-center"
                    key={membership.id}
                    disabled={isCurrent}
                  >
                    {membership.user_email}
                    {isCurrent ? null : (
                      <DeleteMembershipButton membership={membership} />
                    )}
                  </ListGroup.Item>
                );
              })}
            </ListGroup>
          </>
        )}
      </Await>
    </React.Suspense>
  );
}
