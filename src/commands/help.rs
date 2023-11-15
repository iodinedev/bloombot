use crate::Context;
use anyhow::Result;

/// Show help menu
#[poise::command(slash_command)]
pub async fn help(
	ctx: Context<'_>,
	#[description = "Specific command to show help about"]
	#[autocomplete = "poise::builtins::autocomplete_command"]
	command: Option<String>,
) -> Result<()> {

	poise::builtins::help(
		ctx,
		command.as_deref(),
		poise::builtins::HelpConfiguration {
			ephemeral: true,
			..Default::default()
		},
	)
	.await?;

	Ok(())
}
