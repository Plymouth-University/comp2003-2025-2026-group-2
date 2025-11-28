<script lang="ts">
    let logsTemplates = $state([
    {id:1, date: '10/10/2025', label: 'FREEZER TEMPERATURES', frequency:'Weekly'},
    {id:2, date: '10/10/2025', label: 'DISHWASHER TEMPERATURES', frequency:'Weekly'},
    {id:3, date: '10/10/2025', label: 'DISHWASHER CLEANLINESS', frequency:'Daily'},
    {id:4, date: '10/10/2025', label: 'FOOD DATE CODES CHECKED', frequency:'Daily'},
    {id:5, date: '10/10/2025', label: 'MIXER CLEANLINESS', frequency:'Daily'}
]);
</script>

<style>

    .userBox-rows{
        padding:3%;
        
    }
</style>


<main>
    <div class="min-h-screen" style="background-color: #F8F8F8;">
        <div class="flex-1 overflow-auto gap-8">
            <div class="max-w-screen mx-auto">
                {#key logsTemplates}
                    {#each logsTemplates as item}
                    <div class="flex flex-auto flex-row h-30 border-b-2 border-t-2 content-between bg-white overflow-x-none text-2xl" style="margin-top: -2px; display:flex;">
                        <div class="userBox-rows"><b>{item.date}</b></div>
                        <div class="userBox-rows">{item.label}</div>
                        <div class="userBox-rows">-</div>
                        <div class="userBox-rows" style="flex:1">{item.frequency}</div>
                        <button class="userBox-rows" style="cursor:pointer; font-size:35px;">&#128393;</button>
                        <button class="userBox-rows" style="cursor:pointer; font-size:35px;">&#128438;</button>
                    </div>
                    {/each}
                {/key}
            </div>
        </div>
    </div>
</main>