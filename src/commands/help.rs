use crate::Context;
use anyhow::Result;

/// Show the help menu
/// 
/// Shows the help menu.
#[poise::command(slash_command)]
pub async fn help(
	ctx: Context<'_>,
	#[description = "Specific command to show help about"]
	// Disabling autocomplete until menu is displayed dynamically based on permissions.
	// #[autocomplete = "poise::builtins::autocomplete_command"]
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
