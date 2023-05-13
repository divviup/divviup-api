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
}

export interface Account {
  name: string;
  id: string;
  created_at: string;
  updated_at: string;
}
export interface PartialAccount {
  name: string;
}

export interface Membership {
  user_email: string;
  account_id: string;
  id: string;
  created_at: string;
}

type VdafDefinition =
  | { type: "sum"; bits: number }
  | { type: "count" }
  | { type: "histogram"; buckets: number[] };

export interface Task {
  id: string;
  name: string;
  leader_url: string;
  helper_url: string;
  vdaf: VdafDefinition;
  min_batch_size: number;
  time_precision_seconds: number;
  report_count?: number;
  aggregate_collection_count?: number;
  account_id: string;
  created_at: string;
  updated_at: string;
  expiration: string | null;
  is_leader: boolean;
  max_batch_size: number | null;
}

export type NewTask = Omit<
  Task,
  | "id"
  | "report_count"
  | "aggregate_collection_count"
  | "account_id"
  | "created_at"
  | "updated_at"
> & {
  hpke_config: HpkeConfig;
  partner_url: string;
};

export interface HpkeConfig {
  id: number;
  kem_id: number;
  kdf_id: number;
  aead_id: number;
  public_key: string;
}

export interface UpdateTask {
  name: string;
}

export interface CreateMembership {
  user_email: string;
}

const mime = "application/vnd.divviup+json;version=0.1";

export class ApiClient {
  private client?: Promise<AxiosInstance> | AxiosInstance;
  currentUser?: User;

  static async fetchBaseUrl(): Promise<URL> {
    let url = new URL(window.location.href);
    url.pathname = "/api_url";
    let contents = await axios.get(url.toString());
    return new URL(await contents.data);
  }

  private async buildClient(): Promise<AxiosInstance> {
    let baseUrl = await ApiClient.fetchBaseUrl();
    return axios.create({
      baseURL: baseUrl.toString(),
      withCredentials: true,
      headers: {
        "Content-Type": mime,
        Accept: mime,
      },
      validateStatus(status) {
        return status >= 200 && status < 500;
      },
    });
  }

  private async populateClient(): Promise<AxiosInstance> {
    if (!this.client) {
      this.client = this.buildClient();
    }
    return this.client;
  }

  async loginUrl(): Promise<string> {
    return (await this.populateClient()).getUri({ url: "/login" });
  }

  async logoutUrl(): Promise<string> {
    return (await this.populateClient()).getUri({ url: "/logout" });
  }

  isLoggedIn(): boolean {
    return !!this.currentUser;
  }

  async getCurrentUser(): Promise<User> {
    console.log("GETTING CURRENT USER");
    if (this.currentUser) {
      return this.currentUser;
    }
    let client = await this.populateClient();
    let res = await client.get("/api/users/me");
    this.currentUser = res.data as User;
    return this.currentUser;
  }

  private async get(path: string): Promise<AxiosResponse> {
    let client = await this.populateClient();
    return client.get(path);
  }

  private async post(path: string, body: unknown): Promise<AxiosResponse> {
    let client = await this.populateClient();
    return client.post(path, body);
  }

  private async delete(path: string): Promise<AxiosResponse> {
    let client = await this.populateClient();
    return client.delete(path);
  }

  private async patch(path: string, body: unknown): Promise<AxiosResponse> {
    let client = await this.populateClient();
    return client.patch(path, body);
  }

  async accounts(): Promise<Account[]> {
    let res = await this.get("/api/accounts");
    return res.data as Account[];
  }

  async account(id: string): Promise<Account> {
    let res = await this.get(`/api/accounts/${id}`);
    return res.data as Account;
  }

  async createAccount(account: PartialAccount): Promise<Account> {
    const res = await this.post("/api/accounts", account);
    return res.data as Account;
  }

  async updateAccount(id: string, account: PartialAccount): Promise<Account> {
    const res = await this.patch(`/api/accounts/${id}`, account);
    return res.data as Account;
  }

  async accountMemberships(accountId: string): Promise<Membership[]> {
    const res = await this.get(`/api/accounts/${accountId}/memberships`);
    return res.data as Membership[];
  }

  async createMembership(
    accountId: string,
    membership: CreateMembership
  ): Promise<Membership | { error: ValidationErrorsFor<CreateMembership> }> {
    const res = await this.post(
      `/api/accounts/${accountId}/memberships`,
      membership
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

  async updateTask(
    taskId: string,
    task: UpdateTask
  ): Promise<Task | { error: ValidationErrorsFor<UpdateTask> }> {
    const res = await this.patch(`/api/tasks/${taskId}`, task);
    switch (res.status) {
      case 201:
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
    task: NewTask
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
}

function errorToMessage({ message, code, params }: ValidationError) {
  if (message) return message;
  if (code === "required") {
    return "is required";
  } else if (code === "length") {
    if ("min" in params && "max" in params)
      return `length must be between ${params.min} and ${params.max}`;
    if ("min" in params) return `length must be greater than ${params.min}`;
    if ("max" in params) return `length must be less than ${params.max}`;
  } else if (code === "range") {
    if ("min" in params) return `must be greater than ${params.min}`;
    if ("max" in params) return `must be less than ${params.max}`;
  } else {
    console.log({ code, params });
    return code;
  }
}

function errorsToMessage(validationErrors: ValidationError[]) {
  return validationErrors.map(errorToMessage).join(", ");
}

function formikErrors_(v: ValidationErrors): FormikLikeErrors {
  let o = {} as FormikLikeErrors;
  for (let key in v) {
    let e = v[key] as ValidationError[] | ValidationErrors;
    if (Array.isArray(e)) {
      o[key] = errorsToMessage(e);
    } else if (typeof e === "object" && e !== null) {
      o[key] = formikErrors_(e);
    }
  }

  return o;
}

export function formikErrors<T extends object>(
  v: ValidationErrorsFor<T>
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
  params: { value: unknown };
}

export default ApiClient;
