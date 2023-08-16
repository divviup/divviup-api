import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import {
  Await,
  useLoaderData,
  useRouteLoaderData,
  Form,
  useSubmit,
} from "react-router-dom";
import React, { Suspense, useState } from "react";
import { Membership, User } from "../ApiClient";
import { Button, FormControl } from "react-bootstrap";
import { PersonSlash, PersonAdd, People } from "react-bootstrap-icons";
import Modal from "react-bootstrap/Modal";
import { AccountBreadcrumbs, WithAccount } from "../util";
import Placeholder from "react-bootstrap/Placeholder";

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>Memberships</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

export default function Memberships() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <MembershipList />
        </Col>
      </Row>
    </>
  );
}

function DeleteMembershipButton({ membership }: { membership: Membership }) {
  const submit = useSubmit();
  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);

  const deleteMembership = React.useCallback(() => {
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
  const [email, setEmail] = React.useState("");

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
              [setEmail],
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

function MembershipList() {
  const { memberships } = useLoaderData() as {
    memberships: Promise<Membership[]>;
  };
  const { currentUser } = useRouteLoaderData("currentUser") as {
    currentUser: Promise<User>;
  };

  return (
    <>
      <h1>
        <People />{" "}
        <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
          <WithAccount>{(account) => account.name}</WithAccount>
        </Suspense>{" "}
        Members
      </h1>
      <AddMembershipForm />
      <ListGroup>
        <Suspense>
          <Await resolve={memberships}>
            {(memberships: Membership[]) =>
              memberships.map((membership) => (
                <Suspense
                  key={membership.id}
                  fallback={<MembershipItem membership={membership} />}
                >
                  <Await resolve={currentUser}>
                    {(currentUser: User) => (
                      <MembershipItem
                        membership={membership}
                        current={membership.user_email === currentUser.email}
                      />
                    )}
                  </Await>
                </Suspense>
              ))
            }
          </Await>
        </Suspense>
      </ListGroup>
    </>
  );
}

function MembershipItem({
  membership,
  current = true,
}: {
  membership: Membership;
  current?: boolean;
}) {
  return (
    <ListGroup.Item
      className="d-flex justify-content-between align-items-center"
      key={membership.id}
      disabled={current}
    >
      {membership.user_email}

      {current ? null : <DeleteMembershipButton membership={membership} />}
    </ListGroup.Item>
  );
}
