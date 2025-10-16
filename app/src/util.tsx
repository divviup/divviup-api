import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import React, { Suspense } from "react";
import { Await, useRouteLoaderData, useLoaderData } from "react-router-dom";
import { Account } from "./ApiClient";
import Placeholder from "react-bootstrap/Placeholder";
import { Button, OverlayTrigger, Tooltip } from "react-bootstrap";
import {
  ClipboardCheckFill,
  Clipboard,
  BoxArrowUpRight,
} from "react-bootstrap-icons";

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

export function Copy({
  children,
  clipboardContents,
}: {
  children(copy: undefined | (() => void), copied: boolean): React.ReactElement;
  clipboardContents: string;
}) {
  const [copied, setCopied] = React.useState(false);
  const copy = React.useCallback(() => {
    navigator.clipboard.writeText(clipboardContents).then(() => {
      setCopied(true);
    });
  }, [setCopied, clipboardContents]);

  if ("clipboard" in navigator) {
    return (
      <OverlayTrigger
        overlay={<Tooltip>{copied ? "Copied!" : "Click to copy"}</Tooltip>}
      >
        {children(copy, copied)}
      </OverlayTrigger>
    );
  } else {
    return children(undefined, false);
  }
}

export function CopyCode({ code }: { code: string }) {
  return (
    <Copy clipboardContents={code}>
      {(copy, copied) =>
        copy ? (
          <span onClick={copy} style={{ cursor: "pointer" }}>
            <code className="user-select-all">{code}</code>{" "}
            <Button size="sm" variant="outline-secondary" className="ml-auto">
              {copied ? <ClipboardCheckFill /> : <Clipboard />}
            </Button>
          </span>
        ) : (
          <code className="user-select-all">{code}</code>
        )
      }
    </Copy>
  );
}

export function useLoaderPromise<T>(key: string, initialState: T): T {
  return usePromise(
    (useLoaderData() as { [k: string]: Promise<T> })[key],
    initialState,
  );
}

export function usePromise<T>(promise: PromiseLike<T>, initialState: T): T {
  const [state, setState] = React.useState<T>(initialState);
  React.useEffect(() => {
    promise.then((value) => setState(value));
  }, [promise]);
  return state;
}

/**
 * This custom hook allows using the results of three promises. At first, the
 * initial value is returned. Once all promises are ready, their resolved values
 * are passed to the `then` function, and the result is returned.
 *
 * Note that the `then` function should be either defined outside of any render
 * function or passed through `useCallback()`, to avoid spurious renders.
 * @param promise1 First promise
 * @param promise2 Second promise
 * @param promise3 Third promise
 * @param then Function
 * @param initialState Value to be returned until promises are resolved
 * @returns `initialState` or output of `then`
 */
export function usePromiseAll3<U, P1, P2, P3>(
  promise1: P1,
  promise2: P2,
  promise3: P3,
  then: (
    outputs: [Awaited<P1>, Awaited<P2>, Awaited<P3>],
  ) => U | PromiseLike<U>,
  initialState: U,
): U {
  return usePromise(
    React.useMemo(
      () => Promise.all([promise1, promise2, promise3]).then(then),
      [promise1, promise2, promise3, then],
    ),
    initialState,
  );
}

export function OutLink({
  href,
  children,
}: {
  href: string;
  children: React.ReactNode;
}) {
  return (
    <a href={href} target="_blank" rel="noreferrer" className="icon-link">
      {children}
      <BoxArrowUpRight />
    </a>
  );
}

export const numberFormat = Intl.NumberFormat();
