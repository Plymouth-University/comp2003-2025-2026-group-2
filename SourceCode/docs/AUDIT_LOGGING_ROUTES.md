# Audit Logging Route Matrix

This document tracks which API routes currently emit security/audit logs.

Legend:
- Yes: route emits audit/security logs (directly or via service)
- No: route currently does not emit audit/security logs

## Authentication and User

| Method | Route | Logging |
|---|---|---|
| POST | `/auth/register` | Yes |
| POST | `/auth/login` | Yes |
| POST | `/auth/verify` | Yes |
| GET | `/auth/me` | No |
| PUT | `/auth/profile` | Yes |
| POST | `/auth/password/request-reset` | Yes |
| POST | `/auth/password/reset` | Yes |
| POST | `/auth/profile-picture` | No |
| DELETE | `/auth/profile-picture` | No |
| GET | `/auth/profile-picture/{user_id}` | No |

## OAuth

| Method | Route | Logging |
|---|---|---|
| GET | `/auth/google/initiate` | No |
| GET | `/auth/google/callback` | Yes |
| POST | `/auth/google/link` | Yes |
| POST | `/auth/google/link/confirm` | Yes |
| DELETE | `/auth/google/unlink` | Yes |

## Invitations

| Method | Route | Logging |
|---|---|---|
| POST | `/auth/invitations/send` | Yes |
| POST | `/auth/invitations/accept` | Yes |
| PUT | `/auth/invitations/cancel` | Yes |
| GET | `/auth/invitations/details` | No |
| GET | `/auth/invitations/pending` | No |

## Company Members

| Method | Route | Logging |
|---|---|---|
| GET | `/auth/company/members` | No |
| PUT | `/auth/admin/update-member` | Yes |
| DELETE | `/auth/admin/remove-member` | Yes |

## Passkeys

| Method | Route | Logging |
|---|---|---|
| POST | `/auth/passkey/register/start` | Yes |
| POST | `/auth/passkey/register/finish` | Yes |
| POST | `/auth/passkey/login/start` | Yes |
| POST | `/auth/passkey/login/discoverable/start` | Yes |
| POST | `/auth/passkey/login/finish` | Yes |
| POST | `/auth/passkey/login/discoverable/finish` | Yes |
| GET | `/auth/passkeys` | No |
| DELETE | `/auth/passkeys/{passkey_id}` | Yes |

## Branches

| Method | Route | Logging |
|---|---|---|
| POST | `/auth/company/branches` | Yes |
| GET | `/auth/company/branches` | No |
| PUT | `/auth/company/branches` | Yes |
| POST | `/auth/company/branches/request-deletion` | Yes |
| POST | `/auth/company/branches/confirm-deletion` | Yes |

## Company

| Method | Route | Logging |
|---|---|---|
| POST | `/companies/{company_id}/logo` | Yes |
| GET | `/companies/{company_id}/logo` | No |
| DELETE | `/companies/{company_id}/logo` | Yes |
| GET | `/companies/{company_id}` | No |
| PUT | `/companies/{company_id}` | Yes |
| POST | `/companies/{company_id}/export` | Yes |
| GET | `/companies/{company_id}/export/download/{filename}` | Yes |
| DELETE | `/companies/{company_id}` | Yes |
| GET | `/companies/{company_id}/validate-deletion-token` | Yes |
| POST | `/companies/{company_id}/confirm-deletion` | Yes |

## Logs and Templates

| Method | Route | Logging |
|---|---|---|
| POST | `/logs/templates` | No |
| GET | `/logs/templates` | No |
| GET | `/logs/templates/all` | No |
| PUT | `/logs/templates/update` | No |
| PUT | `/logs/templates/rename` | No |
| DELETE | `/logs/templates` | No |
| GET | `/logs/templates/versions` | No |
| POST | `/logs/templates/versions/restore` | No |
| GET | `/logs/entries/due` | No |
| POST | `/logs/entries` | No |
| GET | `/logs/entries` | No |
| GET | `/logs/admin/entries` | No |
| GET | `/logs/entries/{entry_id}` | No |
| PUT | `/logs/entries/{entry_id}` | No |
| DELETE | `/logs/entries/{entry_id}` | No |
| POST | `/logs/entries/{entry_id}/submit` | No |
| POST | `/logs/entries/{entry_id}/unsubmit` | No |

## Clock, LLM, Health, Security Log Read

| Method | Route | Logging |
|---|---|---|
| POST | `/clock/in` | No |
| POST | `/clock/out` | No |
| GET | `/clock/status` | No |
| GET | `/clock/company` | No |
| POST | `/llm/generate-layout` | No |
| GET | `/health/database` | No |
| GET | `/health/slow-queries` | No |
| GET | `/health/index-usage` | No |
| GET | `/health/table-sizes` | No |
| GET | `/security/logs` | No (read endpoint) |
| GET | `/security/logs/export` | No (read endpoint) |
