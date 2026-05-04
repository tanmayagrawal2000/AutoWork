import discord
from discord.ext import commands
import os
import subprocess
import json
import time
import bot_config

# Path to the data directory relative to this script
DATA_DIR = os.path.join(os.path.dirname(__file__), "..", "data")
STATE_FILE = os.path.join(DATA_DIR, "state.json")
PAUSE_FILE = os.path.join(DATA_DIR, "pause_state.json")

class BranchSelect(discord.ui.Select):
    def __init__(self, branches):
        options = []
        for b in branches[:25]:  # Discord limit is 25 options
            is_current = b.startswith('* ')
            branch_name = b.replace('* ', '', 1)
            desc = "Currently active branch" if is_current else f"Switch to {branch_name}"
            options.append(discord.SelectOption(
                label=branch_name, 
                description=desc, 
                default=is_current,
                emoji="📌" if is_current else None
            ))
        super().__init__(placeholder='Select a branch to switch to...', min_values=1, max_values=1, options=options)

    async def callback(self, interaction: discord.Interaction):
        branch = self.values[0]
        project_root = os.path.join(os.path.dirname(__file__), "..")
        
        try:
            process = subprocess.Popen(
                ["git", "checkout", branch],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            stdout, stderr = process.communicate()
            
            embed = interaction.message.embeds[0]
            if process.returncode == 0:
                embed.set_footer(text=f"✅ Successfully switched to branch: {branch}")
            else:
                embed.set_footer(text=f"❌ Failed to switch to: {branch}")
                
            await interaction.response.edit_message(embed=embed, view=DashboardView())
        except Exception as e:
            embed = interaction.message.embeds[0]
            embed.set_footer(text=f"⚠️ Error executing git command: {e}")
            await interaction.response.edit_message(embed=embed, view=DashboardView())

class BranchSelectView(discord.ui.View):
    def __init__(self, branches):
        super().__init__()
        self.add_item(BranchSelect(branches))
        
        # Add a Cancel button to return to the main dashboard
        cancel_btn = discord.ui.Button(label="Cancel", style=discord.ButtonStyle.secondary, emoji="❌")
        async def cancel_callback(interaction: discord.Interaction):
            embed = interaction.message.embeds[0]
            embed.set_footer(text="Action cancelled.")
            await interaction.response.edit_message(embed=embed, view=DashboardView())
        cancel_btn.callback = cancel_callback
        self.add_item(cancel_btn)


class CustomPauseModal(discord.ui.Modal, title='Custom Pause Duration'):
    def __init__(self, original_message, view_to_restore):
        super().__init__()
        self.original_message = original_message
        self.view_to_restore = view_to_restore
        
    hours = discord.ui.TextInput(
        label='Enter duration in hours (e.g., 5.5)',
        style=discord.TextStyle.short,
        placeholder='12',
        required=True,
        max_length=5,
    )

    async def on_submit(self, interaction: discord.Interaction):
        try:
            delay_hours = float(self.hours.value)
            delay_seconds = int(delay_hours * 3600)
            paused_until = time.time() + delay_seconds
            
            os.makedirs(os.path.dirname(PAUSE_FILE), exist_ok=True)
            with open(PAUSE_FILE, 'w') as f:
                json.dump({"paused_until": paused_until}, f)
                
            embed = self.original_message.embeds[0]
            embed.set_footer(text=f"⏸️ Scraper paused for {delay_hours} hours.")
            await interaction.response.edit_message(embed=embed, view=self.view_to_restore)
        except ValueError:
            await interaction.response.send_message("Invalid number of hours provided. Please use a number.", ephemeral=True)

class PauseSelect(discord.ui.Select):
    def __init__(self):
        options = [
            discord.SelectOption(label="30 Minutes", description="Pause the scraper for 30 minutes", value="1800", emoji="⏳"),
            discord.SelectOption(label="1 Hour", description="Pause the scraper for 1 hour", value="3600", emoji="🕐"),
            discord.SelectOption(label="2 Hours", description="Pause the scraper for 2 hours", value="7200", emoji="🕑"),
            discord.SelectOption(label="Custom Time", description="Enter a custom duration in hours", value="custom", emoji="✍️"),
        ]
        super().__init__(placeholder='Select pause duration...', min_values=1, max_values=1, options=options)

    async def callback(self, interaction: discord.Interaction):
        if self.values[0] == "custom":
            await interaction.response.send_modal(CustomPauseModal(interaction.message, DashboardView()))
            return

        delay = int(self.values[0])
        paused_until = time.time() + delay
        
        os.makedirs(os.path.dirname(PAUSE_FILE), exist_ok=True)
        with open(PAUSE_FILE, 'w') as f:
            json.dump({"paused_until": paused_until}, f)
            
        embed = interaction.message.embeds[0]
        embed.set_footer(text=f"⏸️ Scraper paused for {delay // 60} minutes.")
        await interaction.response.edit_message(embed=embed, view=DashboardView())

class PauseSelectView(discord.ui.View):
    def __init__(self):
        super().__init__()
        self.add_item(PauseSelect())
        cancel_btn = discord.ui.Button(label="Cancel", style=discord.ButtonStyle.secondary, emoji="❌")
        async def cancel_callback(interaction: discord.Interaction):
            embed = interaction.message.embeds[0]
            embed.set_footer(text="Action cancelled.")
            await interaction.response.edit_message(embed=embed, view=DashboardView())
        cancel_btn.callback = cancel_callback
        self.add_item(cancel_btn)

class DashboardView(discord.ui.View):
    def __init__(self):
        super().__init__(timeout=None) # Persistent view

    @discord.ui.button(label="Reset State", style=discord.ButtonStyle.danger, custom_id="btn_reset", emoji="🗑️")
    async def reset_state(self, interaction: discord.Interaction, button: discord.ui.Button):
        embed = interaction.message.embeds[0]
        if os.path.exists(STATE_FILE):
            try:
                os.remove(STATE_FILE)
                embed.set_footer(text="✅ state.json deleted. Next run requires fresh login.")
            except Exception as e:
                embed.set_footer(text=f"❌ Failed to delete state.json: {e}")
        else:
            embed.set_footer(text="⚠️ state.json does not exist. Already clean.")
            
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Run Scraper", style=discord.ButtonStyle.success, custom_id="btn_run", emoji="▶️")
    async def run_scraper(self, interaction: discord.Interaction, button: discord.ui.Button):
        script_name = "run_scraper.bat" if os.name == 'nt' else "run_scraper.sh"
        script_path = os.path.join(os.path.dirname(__file__), "..", "scripts", script_name)
        embed = interaction.message.embeds[0]
        
        try:
            subprocess.Popen(
                [script_path, "--headless"],
                cwd=os.path.join(os.path.dirname(__file__), ".."),
                creationflags=subprocess.CREATE_NEW_CONSOLE if os.name == 'nt' else 0
            )
            embed.set_footer(text="🚀 Scraper started in the background...")
        except Exception as e:
            embed.set_footer(text=f"❌ Failed to start scraper: {e}")
            
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Pause Status", style=discord.ButtonStyle.secondary, custom_id="btn_status", emoji="⏱️")
    async def pause_status(self, interaction: discord.Interaction, button: discord.ui.Button):
        embed = interaction.message.embeds[0]
        if os.path.exists(PAUSE_FILE):
            try:
                with open(PAUSE_FILE, 'r') as f:
                    data = json.load(f)
                    remaining = int(data.get('paused_until', 0) - time.time())
                    if remaining > 0:
                        mins = remaining // 60
                        embed.set_footer(text=f"⏸️ Scraper is PAUSED. Resumes in ~{mins} minutes.")
                    else:
                        embed.set_footer(text="▶️ Scraper is ACTIVE (Pause expired).")
            except Exception as e:
                embed.set_footer(text="⚠️ Error reading pause state.")
        else:
            embed.set_footer(text="▶️ Scraper is ACTIVE.")
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Pause Scraper", style=discord.ButtonStyle.secondary, custom_id="btn_pause", emoji="⏸️")
    async def pause_scraper(self, interaction: discord.Interaction, button: discord.ui.Button):
        embed = interaction.message.embeds[0]
        embed.set_footer(text="Select pause duration from the dropdown below:")
        await interaction.response.edit_message(embed=embed, view=PauseSelectView())

    @discord.ui.button(label="Resume Scraper", style=discord.ButtonStyle.secondary, custom_id="btn_resume", emoji="▶️")
    async def resume_scraper(self, interaction: discord.Interaction, button: discord.ui.Button):
        embed = interaction.message.embeds[0]
        if os.path.exists(PAUSE_FILE):
            try:
                os.remove(PAUSE_FILE)
                embed.set_footer(text="▶️ Scraper RESUMED successfully.")
            except Exception as e:
                embed.set_footer(text=f"❌ Failed to resume scraper: {e}")
        else:
            embed.set_footer(text="▶️ Scraper is already active (no pause file).")
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Switch Branch", style=discord.ButtonStyle.primary, custom_id="btn_branch", emoji="🔀")
    async def switch_branch(self, interaction: discord.Interaction, button: discord.ui.Button):
        project_root = os.path.join(os.path.dirname(__file__), "..")
        embed = interaction.message.embeds[0]
        
        try:
            process = subprocess.Popen(
                ["git", "branch"],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            stdout, _ = process.communicate()
            if process.returncode == 0 and stdout:
                branches = [b.strip() for b in stdout.split('\n') if b.strip()]
                if not branches:
                    embed.set_footer(text="⚠️ No branches found.")
                    await interaction.response.edit_message(embed=embed, view=self)
                    return
                
                embed.set_footer(text="Select a branch from the dropdown below:")
                await interaction.response.edit_message(embed=embed, view=BranchSelectView(branches))
            else:
                embed.set_footer(text="❌ Failed to fetch branches.")
                await interaction.response.edit_message(embed=embed, view=self)
        except Exception as e:
            embed.set_footer(text=f"⚠️ Error: {e}")
            await interaction.response.edit_message(embed=embed, view=self)


class AutoWorkBot(commands.Bot):
    def __init__(self):
        intents = discord.Intents.default()
        intents.message_content = True
        super().__init__(command_prefix="!", intents=intents)

    async def setup_hook(self):
        # Add the persistent view so buttons work after restarts
        self.add_view(DashboardView())

    async def on_ready(self):
        print(f"Logged in as {self.user} (ID: {self.user.id})")
        print("------")
        
        # Fetch the designated channel
        channel = self.get_channel(bot_config.CHANNEL_ID)
        
        if channel:
            # Optionally, purge old dashboard messages sent by the bot
            try:
                print("Cleaning up old bot messages in the dashboard channel...")
                async for message in channel.history(limit=50):
                    if message.author == self.user:
                        await message.delete()
            except discord.Forbidden:
                print("Note: Bot lacks 'Manage Messages' permission to clean up old messages.")
            except Exception as e:
                print(f"Failed to clear old messages: {e}")
                
            # Send the new dashboard
            embed = discord.Embed(
                title="⚙️ AutoWork Control Panel",
                description="Use the buttons below to manage the Workday scraper.",
                color=discord.Color.blurple()
            )
            await channel.send(embed=embed, view=DashboardView())
            print("Dashboard sent successfully!")
        else:
            print(f"WARNING: Could not find channel with ID {bot_config.CHANNEL_ID}. Make sure the bot is in the server and the ID is correct.")

if __name__ == "__main__":
    if bot_config.DISCORD_TOKEN == "your_bot_token_here":
        print("Please configure your DISCORD_TOKEN and CHANNEL_ID in discord_bot/bot_config.py")
    else:
        bot = AutoWorkBot()
        bot.run(bot_config.DISCORD_TOKEN)
