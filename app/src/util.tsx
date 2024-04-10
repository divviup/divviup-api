import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import React, { Suspense, useRef } from "react";
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

// Adapted from https://github.com/lbfalvy/react-utils/blob/8c6e750bc6a2450c201ac34c2905aecb43b8350a/src/useArray.ts
/**
 * Ensures referential equality of an array whenever its elements are equal.
 * This allows use of an array in a list of dependencies without unnecessary
 * updates, or changed dependency array length warnings from trying to spread
 * an array into a dependency array.
 * @param input Array
 * @returns Memoized array
 */
function useArray<T extends unknown[]>(input: T): T {
  const ref = useRef<T>(input);
  const cur = ref.current;
  if (input.length === cur.length && input.every((v, i) => v === cur[i])) {
    return cur;
  } else {
    ref.current = input;
    return input;
  }
}

export function usePromiseAll<U, T extends unknown[]>(
  promises: [...T],
  then: (arr: { [P in keyof T]: Awaited<T[P]> }) => U | PromiseLike<U>,
  initialState: U,
): U {
  const memoizedPromises = useArray(promises);
  return usePromise(
    React.useMemo(
      () => Promise.all(memoizedPromises).then(then),
      [memoizedPromises, then],
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
