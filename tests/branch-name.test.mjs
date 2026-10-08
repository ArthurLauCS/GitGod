import assert from 'node:assert/strict'
import { test } from 'node:test'
import { BRANCH_NAME } from '../src/lib/branch-name.ts'

test('branch names follow fix|feat_S|C|SC_lowerCamelCase', () => {
  for (const name of ['feat_S_userLogin', 'fix_SC_orderStatus', 'feat_C_x1']) assert.match(name, BRANCH_NAME)
  for (const name of ['feat_S_UserLogin', 'feat_S_user-login', 'feature_S_x', 'fix_X_x', 'fix_S_', 'feat_s_x', 'feat_S_user_login']) assert.doesNotMatch(name, BRANCH_NAME)
})
