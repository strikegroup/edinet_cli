// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

//! 有価証券報告書の章単位の抽出処理。

mod business_overview;
mod company_overview;
mod corporate_information;
mod facilities;
mod financial_information;

pub(super) use business_overview::extract_business_overview;
pub(super) use company_overview::extract_company_overview;
pub(super) use corporate_information::extract_corporate_information;
pub(super) use facilities::extract_facilities;
pub(super) use financial_information::extract_financial_information;
