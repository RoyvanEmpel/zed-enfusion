[BaseContainerProps(configRoot: true)]
class SCR_ParserConfig : Managed
{
	[Attribute("", UIWidgets.Object, "Available commands")]
	protected ref array<ref SCR_BaseRadialCommand> m_aCommands;

	[RplProp(onRplName: "OnGameStateChanged")]
	private SCR_EGameModeState m_eGameState = SCR_EGameModeState.PREGAME;

	[RplRpc(RplChannel.Reliable, RplRcver.Server)]
	protected void RpcAsk_RequestBuildModeProvider()
	{
	}
}

[WorkbenchPluginAttribute(name: "Map Exporter", wbModules: { "WorldEditor" }, shortcut: "Ctrl+`", awesomeFontCode: 0xF338)]
class WorldExporterPlugin : WorkbenchPlugin
{
	[ButtonAttribute("Run Export", true)]
	protected bool ButtonExport()
	{
		return true;
	}
}
