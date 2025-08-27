import axios, { AxiosInstance, AxiosResponse } from "axios";
import { FormikErrors } from "formik";

export interface User {
  email: string;
  email_verified: boolean;
  name: string;
  nickname: string;
  picture: string;
  sub: string;
  updated_at: string;
  admin: boolean;
}

export interface Account {
  name: string;
  id: string;
  created_at: string;
  updated_at: string;
  intends_to_use_shared_aggregators: boolean;
  admin: boolean;
}

export interface NewAccount {
  name: string;
}

export interface UpdateAccount {
  name?: string;
  intends_to_use_shared_aggregators?: boolean;
}

export interface Membership {
  user_email: string;
  account_id: string;
  id: string;
  created_at: string;
}

export interface QueueJob {
  id: string;
  created_at: string;
  updated_at: string;
  scheduled_at: string | null;
  failure_count: number;
  status: "Success" | "Pending" | "Failed";
  job: {
    type: string;
    version: string;
    [key: string]: unknown;
  };
  error_message: { [key: string]: unknown };
  child_id: string | null;
  parent_id: string | null;
}

type VdafDefinition =
  | { type: "sum"; bits: number }
  | { type: "count" }
  | { type: "histogram"; buckets: number[] };

export interface Task {
  id: string;
  name: string;
  leader_aggregator_id: string;
  helper_aggregator_id: string;
  vdaf: VdafDefinition;
  min_batch_size: number;
  time_precision_seconds: number;
  account_id: string;
  created_at: string;
  updated_at: string;
  expiration: string | null;
  max_batch_size: number | null;
  batch_time_window_size_seconds: number | null;
  collector_credential_id: string;
  report_counter_interval_collected: number;
  report_counter_decode_failure: number;
  report_counter_decrypt_failure: number;
  report_counter_expired: number;
  report_counter_outdated_key: number;
  report_counter_success: number;
  report_counter_too_early: number;
  report_counter_task_expired: number;
  report_counter_duplicate_extension: number;
  aggregation_job_counter_success: number;
  aggregation_job_counter_helper_batch_collected: number;
  aggregation_job_counter_helper_report_replayed: number;
  aggregation_job_counter_helper_report_dropped: number;
  aggregation_job_counter_helper_hpke_unknown_config_id: number;
  aggregation_job_counter_helper_hpke_decrypt_failure: number;
  aggregation_job_counter_helper_vdaf_prep_error: number;
  aggregation_job_counter_helper_task_expired: number;
  aggregation_job_counter_helper_invalid_message: number;
  aggregation_job_counter_helper_report_too_early: number;

}

export interface CollectorAuthToken {
  type: string;
  token: string;
}

export type NewTask = Omit<
  Task,
  | "id"
  | "account_id"
  | "created_at"
  | "updated_at"
  | "vdaf"
  | "expiration"
  | "report_counter_interval_collected"
  | "report_counter_decode_failure"
  | "report_counter_decrypt_failure"
  | "report_counter_expired"
  | "report_counter_outdated_key"
  | "report_counter_success"
  | "report_counter_too_early"
  | "report_counter_task_expired"
  | "report_counter_duplicate_extension"
> & {
  vdaf: {
    type: "sum" | "count" | "histogram";
    bits?: number;
    buckets?: number[];
  };
};

export interface UpdateTask {
  name?: string;
  expiration?: string | null;
}

export interface CreateMembership {
  user_email: string;
}

export type Role =
  | "Leader"
  | "Helper"
  | "Either"
  | "leader"
  | "helper"
  | "either";

export interface Aggregator {
  id: string;
  account_id: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  api_url: string;
  dap_url: string;
  role: Role;
  name: string;
  is_first_party: boolean;
  vdafs: string[];
  query_types: string[];
  features: string[];
  protocol: string;
}

export interface NewAggregator {
  name: string;
  api_url: string;
  bearer_token: string;
  is_first_party?: boolean;
}

export interface UpdateAggregator {
  name?: string;
  bearer_token?: string;
}

export interface ApiToken {
  id: string;
  account_id: string;
  token_hash: string;
  created_at: string;
  deleted_at?: string;
  name?: string;
  last_used_at?: string;
}

export interface CollectorCredential {
  id: string;
  hpke_config: {
    id: number;
    kem_id: string;
    kdf_id: string;
    aead_id: string;
    public_key: string;
  };
  created_at: string;
  deleted_at: null | string;
  updated_at: string;
  name: null | string;
  token_hash: null | string;
}

const mime = "application/vnd.divviup+json;version=0.1";

export class ApiClient {
  #client: Promise<AxiosInstance>;
  #currentUser?: Promise<User>;

  constructor() {
    this.#client = this.buildClient();
  }

  static async fetchBaseUrl(): Promise<URL> {
    const url = new URL(window.location.href);
    url.pathname = "/api_url";
    const contents = await axios.get(url.toString());
    return new URL(await contents.data);
  }

  private async buildClient(): Promise<AxiosInstance> {
    const baseUrl = await ApiClient.fetchBaseUrl();
    return axios.create({
      baseURL: baseUrl.toString(),
      withCredentials: true,
      headers: {
        "Content-Type": mime,
        Accept: mime,
      },
      validateStatus(status) {
        return (status >= 200 && status < 300) || status == 400;
      },
    });
  }

  async apiUrl(): Promise<string> {
    return (await this.#client).getUri({ url: "/" });
  }

  async loginUrl(): Promise<string> {
    return (await this.#client).getUri({ url: "/login" });
  }

  async logoutUrl(): Promise<string> {
    return (await this.#client).getUri({ url: "/logout" });
  }

  async getCurrentUser(): Promise<User> {
    const res = await this.get("/api/users/me");
    return res.data as User;
  }

  async currentUser(): Promise<User> {
    return (this.#currentUser ??= this.getCurrentUser());
  }

  private async get(path: string): Promise<AxiosResponse> {
    const client = await this.#client;
    return client.get(path);
  }

  private async post(path: string, body?: unknown): Promise<AxiosResponse> {
    const client = await this.#client;
    return client.post(path, body);
  }

  private async delete(path: string): Promise<AxiosResponse> {
    const client = await this.#client;
    return client.delete(path);
  }

  private async patch(path: string, body: unknown): Promise<AxiosResponse> {
    const client = await this.#client;
    return client.patch(path, body);
  }

  async accounts(): Promise<Account[]> {
    const res = await this.get("/api/accounts");
    return res.data as Account[];
  }

  async account(id: string): Promise<Account> {
    const res = await this.get(`/api/accounts/${id}`);
    return res.data as Account;
  }

  async createAccount(account: NewAccount): Promise<Account> {
    const res = await this.post("/api/accounts", account);
    return res.data as Account;
  }

  async updateAccount(id: string, account: UpdateAccount): Promise<Account> {
    const res = await this.patch(`/api/accounts/${id}`, account);
    return res.data as Account;
  }

  async accountMemberships(accountId: string): Promise<Membership[]> {
    const res = await this.get(`/api/accounts/${accountId}/memberships`);
    return res.data as Membership[];
  }

  async createMembership(
    accountId: string,
    membership: CreateMembership,
  ): Promise<Membership | { error: ValidationErrorsFor<CreateMembership> }> {
    const res = await this.post(
      `/api/accounts/${accountId}/memberships`,
      membership,
    );
    return res.data as Membership;
  }

  async accountTasks(accountId: string): Promise<Task[]> {
    const res = await this.get(`/api/accounts/${accountId}/tasks`);
    return res.data as Task[];
  }

  async task(taskId: string): Promise<Task> {
    const res = await this.get(`/api/tasks/${taskId}`);
    return res.data as Task;
  }

  async deleteMembership(membershipId: string): Promise<null> {
    await this.delete(`/api/memberships/${membershipId}`);
    return null;
  }

  async accountAggregators(accountId: string): Promise<Aggregator[]> {
    const res = await this.get(`/api/accounts/${accountId}/aggregators`);
    return res.data as Aggregator[];
  }

  async createAggregator(
    accountId: string,
    aggregator: NewAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<NewAggregator> }> {
    const res = await this.post(
      `/api/accounts/${accountId}/aggregators`,
      aggregator,
    );
    switch (res.status) {
      case 201:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<NewAggregator>;
        };
      default:
        throw res;
    }
  }

  async updateAggregator(
    aggregatorId: string,
    aggregator: UpdateAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<UpdateAggregator> }> {
    const res = await this.patch(
      `/api/aggregators/${aggregatorId}`,
      aggregator,
    );
    switch (res.status) {
      case 200:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<UpdateAggregator>;
        };
      default:
        throw res;
    }
  }

  async deleteAggregator(aggregatorId: string): Promise<null> {
    await this.delete(`/api/aggregators/${aggregatorId}`);
    return null;
  }

  async aggregator(aggregatorId: string): Promise<Aggregator> {
    const res = await this.get(`/api/aggregators/${aggregatorId}`);
    return res.data as Aggregator;
  }

  async sharedAggregators(): Promise<Aggregator[]> {
    const res = await this.get("/api/aggregators");
    return res.data as Aggregator[];
  }

  async createSharedAggregator(
    aggregator: NewAggregator,
  ): Promise<Aggregator | { error: ValidationErrorsFor<NewAggregator> }> {
    const res = await this.post(`/api/aggregators`, aggregator);
    switch (res.status) {
      case 201:
        return res.data as Aggregator;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<NewAggregator>;
        };
      default:
        throw res;
    }
  }

  async updateTask(
    taskId: string,
    task: UpdateTask,
  ): Promise<Task | { error: ValidationErrorsFor<UpdateTask> }> {
    const res = await this.patch(`/api/tasks/${taskId}`, task);
    switch (res.status) {
      case 200:
        return res.data as Task;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<UpdateTask>;
        };
      default:
        throw res;
    }
  }

  async createTask(
    accountId: string,
    task: NewTask,
  ): Promise<Task | { error: ValidationErrorsFor<NewTask> }> {
    const res = await this.post(`/api/accounts/${accountId}/tasks`, task);
    switch (res.status) {
      case 201:
        return res.data as Task;
      case 400:
        return { error: res.data } as { error: ValidationErrorsFor<NewTask> };
      default:
        throw res;
    }
  }

  async deleteTask(taskId: string): Promise<null> {
    await this.delete(`/api/tasks/${taskId}`);
    return null;
  }

  async accountApiTokens(accountId: string): Promise<ApiToken[]> {
    const res = await this.get(`/api/accounts/${accountId}/api_tokens`);
    return res.data as ApiToken[];
  }

  async createApiToken(
    accountId: string,
  ): Promise<ApiToken & { token: string }> {
    const res = await this.post(`/api/accounts/${accountId}/api_tokens`);
    return res.data as ApiToken & { token: string };
  }

  async deleteApiToken(tokenId: string): Promise<null> {
    await this.delete(`/api/api_tokens/${tokenId}`);
    return null;
  }

  async updateApiToken(
    tokenId: string,
    token: { name: string },
  ): Promise<null> {
    await this.patch(`/api/api_tokens/${tokenId}`, token);
    return null;
  }

  async queue(searchParams: URLSearchParams): Promise<QueueJob[]> {
    const res = await this.get(`/api/admin/queue?${searchParams}`);
    return res.data as QueueJob[];
  }

  async queueJob(id: string): Promise<QueueJob> {
    const res = await this.get(`/api/admin/queue/${id}`);
    return res.data as QueueJob;
  }

  async deleteCollectorCredential(collectorCredentialId: string) {
    await this.delete(`/api/collector_credentials/${collectorCredentialId}`);
    return null;
  }

  async updateCollectorCredential(
    collectorCredentialId: string,
    collectorCredential: { name: string },
  ) {
    await this.patch(
      `/api/collector_credentials/${collectorCredentialId}`,
      collectorCredential,
    );
    return null;
  }

  async collectorCredential(
    collectorCredentialId: string,
  ): Promise<CollectorCredential> {
    const res = await this.get(
      `/api/collector_credentials/${collectorCredentialId}`,
    );
    return res.data as CollectorCredential;
  }

  async createCollectorCredential(
    accountId: string,
    collectorCredential: { hpke_config: string; name: string },
  ): Promise<
    | CollectorCredential
    | { error: ValidationErrorsFor<{ hpke_config: string; name: string }> }
  > {
    const res = await this.post(
      `/api/accounts/${accountId}/collector_credentials`,
      collectorCredential,
    );
    switch (res.status) {
      case 201:
        return res.data as CollectorCredential;
      case 400:
        return { error: res.data } as {
          error: ValidationErrorsFor<{ hpke_config: string; name: string }>;
        };
      default:
        throw res;
    }
  }

  async accountCollectorCredentials(
    accountId: string,
  ): Promise<CollectorCredential[]> {
    const res = await this.get(
      `/api/accounts/${accountId}/collector_credentials`,
    );
    return res.data as CollectorCredential[];
  }
}

function errorToMessage({ message, code, params }: ValidationError) {
  if (message) return message;
  if (code === "required") {
    return "is required";
  } else if (code === "url") {
    return "must be a well-formed url";
  } else if (code === "https-url") {
    return "must be a well-formed https:// url";
  } else if (code === "no-first-party") {
    return "one of the aggregators must be operated by Divvi Up";
  } else if (code === "base64") {
    return "must be base64";
  } else if (code === "same") {
    return "must not be the same";
  } else if (code === "token-not-recognized") {
    return "bearer token not recognized";
  } else if (code === "http-error") {
    return "error connecting to url";
  } else if (code === "enum" && Array.isArray(params.values)) {
    return `must be one of these values: ${params.values.join(", ")}`;
  } else if (code === "length") {
    if ("min" in params && "max" in params)
      return `length must be between ${params.min} and ${params.max}`;
    if ("min" in params) return `length must be greater than ${params.min}`;
    if ("max" in params) return `length must be less than ${params.max}`;
  } else if (code === "range") {
    if ("min" in params) return `must be greater than ${params.min}`;
    if ("max" in params) return `must be less than ${params.max}`;
  } else {
    // eslint-disable-next-line no-console
    console.error({ code, params });
    return code;
  }
}

function errorsToMessage(validationErrors: ValidationError[]) {
  return [...new Set(validationErrors.map(errorToMessage))].join(", ");
}

function formikErrors_(v: ValidationErrors): FormikLikeErrors {
  const o = {} as FormikLikeErrors;
  for (const key in v) {
    const e = v[key] as ValidationError[] | ValidationErrors;
    if (Array.isArray(e)) {
      o[key] = errorsToMessage(e);
    } else if (typeof e === "object" && e !== null) {
      o[key] = formikErrors_(e);
    }
  }

  return o;
}

export function formikErrors<T extends object>(
  v: ValidationErrorsFor<T>,
): FormikErrors<T> {
  return formikErrors_(v as ValidationErrors) as unknown as FormikErrors<T>;
}

export interface ValidationErrors {
  [key: string]: ValidationError[] | ValidationErrors;
}

export interface FormikLikeErrors {
  [key: string]: string | FormikLikeErrors;
}

export type ValidationErrorsFor<T extends object> = {
  [K in keyof T]?: T[K] extends object
    ? ValidationErrorsFor<T[K]>
    : ValidationError[];
};

export interface ValidationError {
  code: string;
  message: null | string;
  params: { [key: string]: unknown };
}

export default ApiClient;
