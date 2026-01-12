//! Hardcoded Supabase public configuration for distributed builds.
//!
//! NOTE:
//! - `anon` key is NOT a secret. Security is enforced via RLS + JWT.
//! - Do NOT put service_role key here.

pub const SUPABASE_URL: &str = "https://tgclfnahahemystgvkhc.supabase.co";
pub const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InRnY2xmbmFoYWhlbXlzdGd2a2hjIiwicm9sZSI6ImFub24iLCJpYXQiOjE3Njc4NTk5NzYsImV4cCI6MjA4MzQzNTk3Nn0.SDS55p4p75WMxZW6Gyrx-Ow2-Bf0dB8R2yL8M5-Dnm4";

