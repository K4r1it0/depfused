// Type declarations for internal packages
declare module '@xq9zk7823/design-system' {
  export function init(config: any): { ready: boolean };
  export const VERSION: string;
  export const Header: any;
  export const Footer: any;
  export const Button: any;
  export const Card: any;
}

declare module '@xq9zk7823/api-client' {
  export function init(config: any): { ready: boolean };
  export const VERSION: string;
}

declare module '@xq9zk7823/auth-sdk' {
  export function init(config: any): { ready: boolean };
  export const VERSION: string;
  export const AuthProvider: any;
  export const LoginForm: any;
  export function useAuth(): any;
}

declare module 'company-internal-utils' {
  export function log(msg: string): void;
  export function format(s: string): string;
}
