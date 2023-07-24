import {
  Await,
  useNavigate,
  useNavigation,
  useParams,
  useRouteLoaderData,
  NavigateFunction,
  useLoaderData,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import Button from "react-bootstrap/Button";
import Col from "react-bootstrap/Col";
import Form from "react-bootstrap/Form";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import FormLabel from "react-bootstrap/FormLabel";
import FormSelect from "react-bootstrap/FormSelect";
import React, { ChangeEvent, ChangeEventHandler, Suspense } from "react";
import Row from "react-bootstrap/Row";
import { ApiClientContext } from "../ApiClientContext";
import { LinkContainer } from "react-router-bootstrap";
import ApiClient, {
  Account,
  Aggregator,
  NewTask,
  formikErrors,
} from "../ApiClient";
import { Formik, FormikHelpers, FormikProps } from "formik";
import FormCheck from "react-bootstrap/FormCheck";
import { DateTime } from "luxon";
import FormText from "react-bootstrap/FormText";
import { Alert } from "react-bootstrap";

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newTask: NewTask,
  actions: FormikHelpers<NewTask>,
  navigate: NavigateFunction
) {
  try {
    let task = await apiClient.createTask(accountId, newTask);

    if ("error" in task) {
      actions.setErrors(formikErrors(task.error));
    } else {
      return navigate(`/accounts/${accountId}/tasks/${task.id}`);
    }
  } catch (e) {
    console.error(e);
  }
}

type Props = FormikProps<NewTask> & {
  setFocusedField(k: Field<NewTask>): void;
};

function TaskFormGroup({
  children,
  controlId,
}: {
  children: React.ReactNode;
  controlId?: string;
}) {
  return (
    <FormGroup className="mb-5" controlId={controlId}>
      {children}
    </FormGroup>
  );
}

export default function TaskForm() {
  const { account_id } = useParams();
  const { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  const apiClient = React.useContext(ApiClientContext);
  const navigate = useNavigate();
  const handleSubmit = React.useCallback(
    (values: NewTask, actions: FormikHelpers<NewTask>) =>
      submit(apiClient, account_id as string, values, actions, navigate),
    [apiClient, account_id, navigate]
  );
  const navigation = useNavigation();

  const [focusedField, setFocusedField] = React.useState<Field<NewTask> | null>(
    null
  );

  const focus = React.useCallback(
    (event: React.FocusEvent<HTMLFormElement>) => {
      setFocusedField(event.target.name as unknown as Field<NewTask>);
    },
    [setFocusedField, navigate]
  );

  return (
    <>
      <Breadcrumbs account={account} />
      <Row style={{ height: "calc(100vh - 100px)" }}>
        <Col sm="6" style={{ maxHeight: "100%", overflowY: "auto" }}>
          <Formik
            validateOnChange={false}
            validateOnBlur={false}
            validateOnMount={false}
            initialValues={
              {
                min_batch_size: 100,
                name: "",
                max_batch_size: null,
                time_precision_seconds: 3600,
                expiration: null,
                hpke_config: "",
              } as unknown as NewTask
            }
            onSubmit={handleSubmit}
          >
            {(formikProps) => {
              const props = { ...formikProps, setFocusedField };
              return (
                <Form
                  className="mb-5"
                  method="post"
                  onSubmit={props.handleSubmit}
                  noValidate
                  autoComplete="off"
                  onFocus={focus}
                >
                  <TaskName {...props} />
                  <LeaderAggregator {...props} />
                  <HelperAggregator {...props} />
                  <VdafType {...props} />
                  <VdafDetails {...props} />
                  <QueryType {...props} />
                  <MinBatchSize {...props} />
                  <HpkeConfig {...props} />
                  <TimePrecisionSeconds {...props} />
                  <Expiration {...props} />
                  <TaskFormGroup>
                    <Button
                      variant="primary"
                      type="submit"
                      disabled={navigation.state === "submitting"}
                    >
                      Submit
                    </Button>
                  </TaskFormGroup>
                </Form>
              );
            }}
          </Formik>
        </Col>
        <Col sm={6}>
          {focusedField ? <LongHelpText field={focusedField} /> : null}
        </Col>
      </Row>
    </>
  );
}

type Field<O extends object> = {
  [K in keyof O & string]: O[K] extends object ? `${K}.${Field<O[K]>}` : K;
}[keyof O & string];

const helps: {
  [K in Field<NewTask>]?: {
    title: string;
    short: React.ReactNode;
    long?: React.ReactNode;
  };
} = {
  name: {
    title: "Task Name",
    short:
      "A short name to identify this task in this application. This can be edited.",
  },

  leader_aggregator_id: {
    title: "Leader Aggregator",
    short: "Select an aggregator server to process this metrics task.",
    long: (
      <p>
        The leader aggregator is one of the two non-colluding servers that
        processes metrics tasks. Its role is more resource-intensive than the
        helper&apos;s. One of the two aggregators must be run by Divvi Up, and
        the other must be run by a different organization. To use a self-hosted
        aggregator, you must first add it to your account; it will then appear
        in this list.
      </p>
    ),
  },

  helper_aggregator_id: {
    title: "Helper Aggregator",
    short: "Select an aggregator server to process this metrics task.",
    long: (
      <p>
        The helper aggregator is one of the two non-colluding servers that
        processes metrics tasks. Its role is less resource-intensive than the
        leader&apos;s. One of the two aggregators must be run by Divvi Up, and
        the other must be run by a different organization. To use a self-hosted
        aggregator, you must first add it to your account; it will then appear
        in this list.
      </p>
    ),
  },

  min_batch_size: {
    title: "Minimum Batch Size",
    short: "Minimum number of reports per batch.",
    long: (
      <p>
        This number should be set high enough that the aggregate results over a
        batch do not violate the application&apos;s privacy goals. This is
        determined by a number of factors, including the aggregation function
        used, the population distribution of measurements, the
        importance/sensitivity of the underlying data, and whether client
        attestation is used to prevent{" "}
        <a href="https://www.ietf.org/archive/id/draft-ietf-ppm-dap-05.html#section-7-4.3">
          Sybil attacks
        </a>
        . If differential privacy noise is added, it can simplify selection of a
        minimum batch size.
      </p>
    ),
  },

  "vdaf.type": {
    title: "Function",
    short:
      "Determines the kind of client measurement accepted, and how they are summarized.",
    long: (
      <>
        <p>Selects the aggregation function used by this metrics task.</p>
        <p>The following functions are supported:</p>
        <ul>
          <li>
            Count: Each client measurement is either "true" or "false". The
            aggregate result is the number of "true" measurements.
          </li>
          <li>
            Sum: Each client measurement is an integer number. The aggregate
            result is the sum of the measurements.
          </li>
          <li>
            Histogram: The aggregate result is a list of counters, and each
            client measurement chooses one counter to increment.
          </li>
        </ul>
      </>
    ),
  },

  "vdaf.bits": {
    title: "Measurement Range",
    short: "Selects the bit width and range of valid client measurements.",
    long: (
      <p>
        Determines the range of integers that are accepted as client
        measurements. Note that this only determines the maximum value of
        individual measurements, and not the maximum value of the aggregate
        result (sum of measurements). Regardless of this choice, the aggregate
        result wraps around at about 3.4×10<sup>38</sup>. This parameter affects
        the size of client reports.
      </p>
    ),
  },

  "vdaf.buckets": {
    title: "Histogram",
    short: "Selects the number of histogram buckets or counters.",
    long: (
      <p>
        Determines how many buckets the histogram has. Each client report can
        only add one to a single bucket/counter. This parameter affects the size
        of client reports.
      </p>
    ),
  },

  max_batch_size: {
    title: "Query Type",
    short:
      "Determines how reports are grouped into batches, and what kinds of queries the collector can make.",
    long: (
      <>
        <h5>Time Interval:</h5>
        <p>
          Groups measurements into batches by their client timestamp. Collectors
          may query for aggregate results over (non-overlapping) time intervals.
          Good for identifying temporal patterns in data. If client reports may
          be received late, well after their timestamps, then the collector is
          forced to choose between delaying collection requests or abandoning
          late reports.
        </p>
        <h5>Fixed Size:</h5>
        <p>
          Groups measurements into batches arbitrarily as they arrive. Grants
          more control over batch sizes, because a maximum batch size can be
          set. Good for cases where the report upload rate is unknown or varies
          widely. Temporal patterns in data may be obscured by aggregating
          on-time and late reports together in the same batches.
        </p>
      </>
    ),
  },

  hpke_config: {
    title: "DAP-encoded HPKE file",
    short:
      "The collector's public key. Results will be encrypted using this key.",
    long: (
      <p>
        Upload a binary public key file in DAP "HpkeConfig" format. Do not
        upload the corresponding private key. You will need to use the private
        key when collecting aggregate results, to decrypt results from the
        aggregators.
      </p>
    ),
  },

  time_precision_seconds: {
    title: "Time precision",
    short: "Granularity of client report timestamps.",
    long: (
      <>
        <p>
          All client report timestamps will be rounded down to the previous
          multiple of this duration.
        </p>
        <p>
          If the query type is Time Interval, then query time intervals must
          have a duration at least as long as the time precision, must have a
          duration that is a multiple of the time precision, and must start and
          end on multiples of the time precision as well.
        </p>
      </>
    ),
  },

  expiration: {
    title: "Expiration",
    short: "Optional, pre-scheduled time to decommission this task.",
    long: (
      <p>
        If set, then reports may no longer be uploaded for this task after its
        expiration time.
      </p>
    ),
  },
};

function LongHelpText({ field }: { field: Field<NewTask> }) {
  let help = helps[field];
  if (help && help.long) {
    return (
      <Alert>
        <Alert.Heading>{help.title}</Alert.Heading>
        {help.long}
      </Alert>
    );
  } else {
    return null;
  }
}

function ShortHelpAndLabel({
  htmlFor,
  fieldKey,
  setFocusedField,
}: {
  htmlFor?: string;
  fieldKey: Field<NewTask>;
  setFocusedField(field: Field<NewTask>): void;
}) {
  let help = helps[fieldKey];
  if (help && help.title) {
    return (
      <>
        <FormLabel htmlFor={htmlFor}>{help.title}</FormLabel>
        <ShortHelpText fieldKey={fieldKey} setFocusedField={setFocusedField} />
      </>
    );
  } else return null;
}

function ShortHelpText({
  fieldKey,
  setFocusedField,
}: {
  fieldKey: Field<NewTask>;
  setFocusedField(field: Field<NewTask>): void;
}) {
  const expand = React.useCallback(
    (e: React.MouseEvent) => {
      setFocusedField(fieldKey);
      e.preventDefault();
    },
    [setFocusedField, fieldKey]
  );

  let help = helps[fieldKey];
  if (help) {
    return (
      <div>
        <FormText muted>
          {help.short}
          {help.long ? (
            <>
              {" "}
              <a onClick={expand} href="#">
                more&raquo;
              </a>
            </>
          ) : null}
        </FormText>
      </div>
    );
  } else {
    return null;
  }
}

function LeaderAggregator(props: Props) {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };

  return (
    <TaskFormGroup controlId="leader_aggregator_id">
      <ShortHelpAndLabel
        fieldKey="leader_aggregator_id"
        setFocusedField={props.setFocusedField}
      />

      <FormSelect
        value={props.values.leader_aggregator_id}
        isInvalid={!!props.errors.leader_aggregator_id}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        name="leader_aggregator_id"
      >
        <option />

        <Suspense>
          <Await resolve={aggregators}>
            {(aggregators: Aggregator[]) =>
              aggregators
                .filter((a) => a.role === "Leader" || a.role === "Either")
                .map((aggregator) => (
                  <option key={aggregator.id} value={aggregator.id}>
                    {aggregator.name}
                  </option>
                ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.leader_aggregator_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function HelperAggregator(props: Props) {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };
  return (
    <TaskFormGroup>
      <ShortHelpAndLabel
        fieldKey="helper_aggregator_id"
        setFocusedField={props.setFocusedField}
      />

      <FormSelect
        value={props.values.helper_aggregator_id}
        isInvalid={!!props.errors.helper_aggregator_id}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        name="helper_aggregator_id"
      >
        <option />
        <Suspense>
          <Await resolve={aggregators}>
            {(aggregators: Aggregator[]) =>
              aggregators
                .filter((a) => a.role === "Helper" || a.role === "Either")
                .map((aggregator) => (
                  <option key={aggregator.id} value={aggregator.id}>
                    {aggregator.name}
                  </option>
                ))
            }
          </Await>
        </Suspense>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.helper_aggregator_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function QueryType(props: Props) {
  const {
    setFieldValue,
    values: { max_batch_size, min_batch_size },
  } = props;
  const timeInterval = typeof max_batch_size !== "number";

  const checkboxChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setFieldValue(
        "max_batch_size",
        event.target.value === "fixed" && typeof min_batch_size === "number"
          ? min_batch_size * 2
          : /*jbr: I have no idea what a good
             * default is, but it needs to be
             * greater than min*/
            null
      );
    },
    [setFieldValue, min_batch_size]
  );

  return (
    <TaskFormGroup controlId="querytype">
      <ShortHelpAndLabel
        fieldKey="max_batch_size"
        setFocusedField={props.setFocusedField}
      />
      <FormCheck
        type="radio"
        name="query-type"
        id="query=type-time"
        checked={timeInterval}
        onChange={checkboxChange}
        label="Time Interval"
        value="time"
      />
      <FormCheck
        type="radio"
        name="query-type"
        id="query-type-fixed"
        checked={!timeInterval}
        onChange={checkboxChange}
        label="Fixed Size"
        value="fixed"
      />
      <MaxBatchSize {...props} />
    </TaskFormGroup>
  );
}

function MaxBatchSize(props: Props) {
  const { values, setFieldValue, errors, handleBlur } = props;
  const { max_batch_size, min_batch_size } = values;

  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (event.target.value) {
        setFieldValue("max_batch_size", event.target.valueAsNumber);
      }
    },
    [setFieldValue]
  );

  if (typeof max_batch_size !== "number") return null;
  return (
    <TaskFormGroup>
      <FormLabel>Maximum Batch Size</FormLabel>
      <FormControl
        type="number"
        name="max_batch_size"
        value={max_batch_size}
        onChange={handleChange}
        step={1}
        onBlur={handleBlur}
        min={min_batch_size}
        isInvalid={!!errors.max_batch_size}
      />
      <FormControl.Feedback type="invalid">
        {errors.max_batch_size}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function HpkeConfig({ setFieldValue, errors, setFocusedField }: Props) {
  let reader = React.useMemo(() => {
    let reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") {
        setFieldValue("hpke_config", reader.result.split(",")[1]);
      }
    });
    return reader;
  }, [setFieldValue]);
  let onChange: ChangeEventHandler<HTMLInputElement> = React.useCallback(
    (event) => {
      let files = event.target.files;
      if (files && files[0]) {
        reader.readAsDataURL(files[0]);
      }
    },
    [reader]
  );

  return (
    <TaskFormGroup controlId="hpke_config">
      <ShortHelpAndLabel
        fieldKey="hpke_config"
        setFocusedField={setFocusedField}
      />
      <FormControl
        type="file"
        onChange={onChange}
        isInvalid={!!errors.hpke_config}
      />
      <FormControl.Feedback type="invalid">
        {errors.hpke_config}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function TaskName(props: Props) {
  return (
    <TaskFormGroup controlId="name">
      <ShortHelpAndLabel
        fieldKey="name"
        setFocusedField={props.setFocusedField}
      />
      <FormControl
        type="text"
        name="name"
        autoComplete="off"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.name}
        isInvalid={!!props.errors.name}
        data-1p-ignore
      />
      <FormControl.Feedback type="invalid">
        {props.errors.name}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function MinBatchSize(props: Props) {
  return (
    <TaskFormGroup controlId="min_batch_size">
      <ShortHelpAndLabel
        fieldKey="min_batch_size"
        setFocusedField={props.setFocusedField}
      />
      <FormControl
        type="number"
        name="min_batch_size"
        min="300"
        value={props.values.min_batch_size}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={!!props.errors.min_batch_size}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.min_batch_size}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

const seconds = {
  minute: 60,
  hour: 60 * 60,
  day: 60 * 60 * 24,
  week: 60 * 60 * 24 * 7,
};
type Unit = keyof typeof seconds;
function TimePrecisionSeconds(props: Props) {
  let { setFieldValue } = props;
  const [count, setCount] = React.useState<number | undefined>(undefined);
  const [unit, setUnit] = React.useState<Unit>("minute");

  React.useEffect(() => {
    if (typeof count === "number" && unit in seconds) {
      setFieldValue("time_precision_seconds", seconds[unit] * count);
    } else {
      setFieldValue("time_precision_seconds", undefined);
    }
  }, [count, unit, setFieldValue]);

  const changeUnit = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setUnit(event.target.value as Unit);
    },
    [setUnit]
  );

  const changeCount = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setCount(event.target.valueAsNumber);
    },
    [setCount]
  );

  return (
    <TaskFormGroup>
      <ShortHelpAndLabel
        fieldKey="time_precision_seconds"
        setFocusedField={props.setFocusedField}
      />
      <Row>
        <Col xs="2">
          <FormControl
            type="number"
            value={count || ""}
            id="time-precision-number"
            onChange={changeCount}
            isInvalid={!!props.errors.time_precision_seconds}
          />
        </Col>
        <Col>
          <FormSelect
            value={unit}
            onChange={changeUnit}
            isInvalid={!!props.errors.time_precision_seconds}
            id="time-precision-unit"
          >
            {Object.keys(seconds).map((unit) => (
              <option key={unit} value={unit}>
                {unit}
                {count === 1 ? "" : "s"}
              </option>
            ))}
          </FormSelect>
          <FormControl.Feedback type="invalid">
            {props.errors.time_precision_seconds}
          </FormControl.Feedback>
        </Col>
      </Row>
    </TaskFormGroup>
  );
}

function VdafType(props: Props) {
  return (
    <TaskFormGroup controlId="vdaf.type">
      <ShortHelpAndLabel
        fieldKey="vdaf.type"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        value={props.values.vdaf?.type}
        name="vdaf.type"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={typeof props.errors.vdaf === "string"}
      >
        <option></option>
        <option value="sum">sum</option>
        <option value="histogram">histogram</option>
        <option value="count">count</option>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "string" ? props.errors.vdaf : null}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function VdafDetails(props: Props) {
  switch (props.values.vdaf?.type) {
    case "sum":
      return <SumBits {...props} />;

    case "histogram":
      return <HistogramBucketSelection {...props} />;

    default:
      return <></>;
  }
}

function HistogramBucketSelection(props: Props) {
  let { setFieldValue } = props;
  let [input, setInput] = React.useState(
    props.values.vdaf?.type === "histogram"
      ? (props.values.vdaf?.buckets || []).join(", ")
      : ""
  );

  let cb = React.useCallback(
    (change: ChangeEvent<HTMLInputElement>) => {
      if (/^([0-9]+, *)*[0-9]*$/.test(change.target.value)) {
        if (input.length) {
          setFieldValue(
            "vdaf.buckets",
            input
              .split(/, */)
              .map((n) => parseInt(n, 10))
              .filter((n) => !isNaN(n))
          );
        }
        setInput(change.target.value);
      } else {
        change.stopPropagation();
        change.preventDefault();
      }
    },
    [input, setInput, setFieldValue]
  );

  let blur = React.useCallback(() => {
    let value = input
      .split(/, */)
      .map((n) => parseInt(n, 10))
      .sort((a, b) => a - b);
    value = [...new Set(value)];
    setInput(value.join(", "));
    setFieldValue("vdaf.buckets", value);
  }, [input, setInput, setFieldValue]);

  if (props.values.vdaf?.type !== "histogram") return <></>;
  return (
    <TaskFormGroup controlId="vdaf.buckets">
      <ShortHelpAndLabel
        fieldKey="vdaf.buckets"
        setFocusedField={props.setFocusedField}
      />

      <FormControl
        value={input}
        name="vdaf.buckets"
        onChange={cb}
        onBlur={blur}
        isInvalid={
          typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          !!props.errors.vdaf.buckets
        }
      />
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          props.errors.vdaf.buckets}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}

function SumBits(props: Props) {
  let { setFieldValue } = props;
  let handleChange = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) =>
      setFieldValue("vdaf.bits", parseInt(event.target.value, 10)),
    [setFieldValue]
  );
  if (props.values.vdaf?.type !== "sum") return <></>;

  return (
    <TaskFormGroup controlId="vdaf.bits">
      <ShortHelpAndLabel
        fieldKey="vdaf.bits"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        value={props.values.vdaf?.bits}
        name="vdaf.bits"
        onChange={handleChange}
        onBlur={props.handleBlur}
      >
        {[8, 16, 32, 64].map((i) => (
          <option value={i} key={i}>
            Unsigned {i}-bit integer (0 to{" "}
            {(Math.pow(2, i) - 1).toLocaleString()})
          </option>
        ))}
      </FormSelect>
    </TaskFormGroup>
  );
}

function Expiration(props: Props) {
  let { setFieldValue, values } = props;
  let { expiration } = values;

  const [enabled, setEnabled] = React.useState(false);

  const checkboxChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setEnabled(event.target.checked);
      if (!event.target.checked) setFieldValue("expiration", null);
    },
    [setEnabled, setFieldValue]
  );

  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (event.target.value) {
        let datetime = DateTime.fromISO(event.target.value);
        setFieldValue(
          "expiration",
          datetime.toISO({
            suppressSeconds: true,
            suppressMilliseconds: true,
            includeOffset: true,
          })
        );
      } else {
        setFieldValue("expiration", null);
      }
    },
    [setFieldValue]
  );

  let min = React.useMemo(
    () =>
      DateTime.now().set({ second: 0, millisecond: 0 }).toISO({
        includeOffset: false,
        suppressSeconds: true,
        suppressMilliseconds: true,
      }),
    []
  );

  const formValue = expiration
    ? DateTime.fromISO(expiration)
        .toLocal()
        .set({ second: 0, millisecond: 0 })
        .toISO({
          includeOffset: false,
          suppressSeconds: true,
          suppressMilliseconds: true,
        }) || ""
    : "";

  return (
    <TaskFormGroup controlId="expiration">
      <FormCheck
        type="switch"
        checked={enabled}
        onChange={checkboxChange}
        label="Expiration"
      />
      {enabled ? (
        <FormControl
          type="datetime-local"
          name="expiration"
          value={formValue}
          onChange={handleChange}
          onBlur={props.handleBlur}
          step={60}
          min={min || undefined}
          isInvalid={!!props.errors.expiration}
        />
      ) : null}
      <FormControl.Feedback type="invalid">
        {props.errors.expiration}
      </FormControl.Feedback>
      <ShortHelpText
        fieldKey="expiration"
        setFocusedField={props.setFocusedField}
      />
    </TaskFormGroup>
  );
}

// <FormControl
//   value={props.values.vdaf?.type}
//   name="vdaf.bits"
//   onChange={props.handleChange}
//   onBlur={props.handleBlur}
//   isInvalid={!!props.errors.time_precision_seconds}
// />

function Breadcrumbs({ account }: { account: Promise<Account> }) {
  let { account_id } = useParams();

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
          <LinkContainer to={`/accounts/${account_id}`}>
            <Breadcrumb.Item>
              <React.Suspense fallback={<span>...</span>}>
                <Await resolve={account}>{(account) => account.name}</Await>
              </React.Suspense>
            </Breadcrumb.Item>
          </LinkContainer>

          <LinkContainer to={`/accounts/${account_id}/tasks`}>
            <Breadcrumb.Item>Tasks</Breadcrumb.Item>
          </LinkContainer>
          <Breadcrumb.Item active>new</Breadcrumb.Item>
        </Breadcrumb>
      </Col>
    </Row>
  );
}
